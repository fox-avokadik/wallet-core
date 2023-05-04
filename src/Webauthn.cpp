// Copyright © 2017-2023 Trust Wallet.
//
// This file is part of Trust. The full Trust copyright notice, including
// terms governing use, modification, and redistribution, is contained in the
// file LICENSE at the root of the source code distribution tree.


#include "Cbor.h"

#include <iostream>
#include <utility>
#include <algorithm>

namespace TW::Webauthn {

// https://www.w3.org/TR/webauthn-2/#authenticator-data
struct AuthData {
    Data rpIdHash;
    std::array<std::uint8_t, 1> flagsBuf;
    struct {
        bool up;
        bool uv;
        bool at;
        bool ed;
        std::uint8_t flagsInt;
    } flags;
    std::uint32_t counter;
    Data counterBuf;
    Data aaguid;
    Data credID;
    Data COSEPublicKey;
};

AuthData parseAuthData(const Data& buffer) {
    AuthData authData;

    authData.rpIdHash = Data(buffer.begin(), buffer.begin() + 32);

    auto it = buffer.begin() + 32;
    authData.flagsBuf = { *it };
    ++it;
    std::uint8_t flagsInt = authData.flagsBuf[0];
    authData.flags.up = !!(flagsInt & 0x01);
    authData.flags.uv = !!(flagsInt & 0x04);
    authData.flags.at = !!(flagsInt & 0x40);
    authData.flags.ed = !!(flagsInt & 0x80);
    authData.flags.flagsInt = flagsInt;

    authData.counterBuf = Data(it, it + 4);
    authData.counter = static_cast<std::uint32_t>((authData.counterBuf[0] << 24) |
                                                  (authData.counterBuf[1] << 16) |
                                                  (authData.counterBuf[2] << 8) |
                                                  authData.counterBuf[3]);
    it += 4;

    if (authData.flags.at) {
        authData.aaguid = Data(it, it + 16);
        it += 16;

        std::array<std::uint8_t, 2> credIDLenBuf = { *(it), *(it + 1) };
        std::uint16_t credIDLen = static_cast<std::uint16_t>((credIDLenBuf[0] << 8) |
                                                             credIDLenBuf[1]);
        it += 2;

        authData.credID = Data(it, it + credIDLen);
        it += credIDLen;

        authData.COSEPublicKey = Data(it, buffer.end());
    }

    return authData;
}

Data getPublicKey(const Data& attestationObject) {
    Data authData = TW::Cbor::Decode(attestationObject).getMapElements()[2].second.getBytes();
    AuthData authDataParsed = parseAuthData(authData);

    std::vector<std::pair<TW::Cbor::Decode, TW::Cbor::Decode>> COSEPublicKey = TW::Cbor::Decode(authDataParsed.COSEPublicKey).getMapElements();

    // https://www.w3.org/TR/webauthn-2/#sctn-encoded-credPubKey-examples
    std::string xKey = "-2";
    std::string yKey = "-3";

    auto x = std::find_if(COSEPublicKey.begin(), COSEPublicKey.end(), [&xKey](const auto& p) { return p.first.dumpToString() == xKey; });
    auto y = std::find_if(COSEPublicKey.begin(), COSEPublicKey.end(), [&yKey](const auto& p) { return p.first.dumpToString() == yKey; });

    Data publicKey;
    append(publicKey, x->second.getBytes());
    append(publicKey, y->second.getBytes());
    return publicKey;
}

} // namespace TW::Webauthn