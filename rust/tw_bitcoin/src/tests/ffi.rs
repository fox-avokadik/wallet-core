use crate::brc20::{BRC20TransferInscription, Ticker};
use crate::ffi::{
    taproot_build_and_sign_transaction, tw_build_brc20_inscribe_transfer, tw_build_p2pkh_script,
    tw_build_p2tr_key_path_script, tw_build_p2wpkh_script,
};
use crate::tests::p2pkh::ALICE_WIF;
use crate::{
    keypair_from_wif, Recipient, TXOutputP2TRScriptPath, TxInputP2PKH, TxOutputP2PKH,
    TxOutputP2TRKeyPath, TxOutputP2WPKH,
};
use bitcoin::PublicKey;
use std::borrow::Cow;
use tw_encoding::hex;
use tw_proto::Bitcoin::Proto::{
    SigningInput, TransactionOutput, TransactionPlan, TransactionVariant, UnspentTransaction,
};

#[test]
fn build_ffi_p2pkh_script() {
    let keypair: secp256k1::KeyPair = keypair_from_wif(ALICE_WIF).unwrap();
    let recipient = Recipient::<PublicKey>::from(keypair);

    let satoshis: u64 = 1_000;

    // Call FFI function.
    let pubkey = recipient.public_key().to_bytes();
    let array =
        unsafe { tw_build_p2pkh_script(satoshis as i64, pubkey.as_ptr(), pubkey.len()).into_vec() };

    let ffi_der: TransactionOutput = tw_proto::deserialize(&array).unwrap();

    // Compare with native call.
    let tx_out = TxOutputP2PKH::new(satoshis, recipient);
    let proto = TransactionOutput {
        value: satoshis as i64,
        script: Cow::from(tx_out.script_pubkey.as_bytes()),
        spendingScript: Cow::default(),
    };

    assert_eq!(ffi_der, proto);
}

#[test]
fn build_ffi_p2wpkh_script() {
    let keypair: secp256k1::KeyPair = keypair_from_wif(ALICE_WIF).unwrap();
    let recipient = Recipient::<PublicKey>::from(keypair);

    let satoshis: u64 = 1_000;

    // Call FFI function.
    let pubkey = recipient.public_key().to_bytes();
    let array = unsafe {
        tw_build_p2wpkh_script(satoshis as i64, pubkey.as_ptr(), pubkey.len()).into_vec()
    };

    let ffi_der: TransactionOutput = tw_proto::deserialize(&array).unwrap();

    // Compare with native call.
    let tx_out = TxOutputP2WPKH::new(satoshis, recipient.try_into().unwrap());
    let proto = TransactionOutput {
        value: satoshis as i64,
        script: Cow::from(tx_out.script_pubkey.as_bytes()),
        spendingScript: Cow::default(),
    };

    assert_eq!(ffi_der, proto);
}

#[test]
fn build_ffi_p2tr_key_path_script() {
    let keypair: secp256k1::KeyPair = keypair_from_wif(ALICE_WIF).unwrap();
    let recipient = Recipient::<PublicKey>::from(keypair);

    let satoshis: u64 = 1_000;

    // Call FFI function.
    let pubkey = recipient.public_key().to_bytes();
    let array = unsafe {
        tw_build_p2tr_key_path_script(satoshis as i64, pubkey.as_ptr(), pubkey.len()).into_vec()
    };

    let ffi_der: TransactionOutput = tw_proto::deserialize(&array).unwrap();

    // Compare with native call.
    let tx_out = TxOutputP2TRKeyPath::new(satoshis, recipient.try_into().unwrap());
    let proto = TransactionOutput {
        value: satoshis as i64,
        script: Cow::from(tx_out.script_pubkey.as_bytes()),
        spendingScript: Cow::default(),
    };

    assert_eq!(ffi_der, proto);
}

#[test]
fn build_ffi_brc20_transfer_script() {
    let keypair: secp256k1::KeyPair = keypair_from_wif(ALICE_WIF).unwrap();
    let recipient = Recipient::<PublicKey>::from(keypair);

    let satoshis: u64 = 1_000;
    let brc20_amount = 20;
    let ticker = "oadf";

    // Call FFI function.
    let tick_buf = ticker.as_bytes();
    let pubkey = recipient.public_key().to_bytes();
    let array = unsafe {
        tw_build_brc20_inscribe_transfer(
            tick_buf.as_ptr(),
            brc20_amount,
            satoshis as i64,
            pubkey.as_ptr(),
            pubkey.len(),
        )
        .into_vec()
    };

    let ffi_der: TransactionOutput = tw_proto::deserialize(&array).unwrap();

    // Compare with native call.
    let transfer = BRC20TransferInscription::new(
        recipient,
        Ticker::new(ticker.to_string()).unwrap(),
        brc20_amount,
    )
    .unwrap();

    let tapscript = transfer.0.recipient().clone();
    let spending_script = transfer.0.envelope.script;

    let tx_out = TXOutputP2TRScriptPath::new(satoshis, &tapscript);
    let proto = TransactionOutput {
        value: satoshis as i64,
        script: Cow::from(tx_out.script_pubkey.as_bytes()),
        spendingScript: Cow::from(spending_script.as_bytes()),
    };

    assert_eq!(ffi_der, proto);
}

#[test]
fn proto_sign_input_p2pkh_output_p2pkh() {
    use crate::tests::p2pkh::{
        ALICE_WIF, BOB_WIF, EXPECTED_RAW_SIGNED, FULL_AMOUNT, SEND_AMOUNT, TXID,
    };

    let alice: secp256k1::KeyPair = keypair_from_wif(ALICE_WIF).unwrap();
    let alice_privkey = alice.secret_bytes();
    let bob = keypair_from_wif(BOB_WIF).unwrap();

    // Note that the Txid must be reversed.
    let txid: Vec<u8> = hex::decode(TXID).unwrap().into_iter().rev().collect();

    // Prepare the scripts.
    // Note that here the input and outputs script are identical (in_script =
    // out_script), the scriptSig/Witness for claiming a transaction is created
    // and set by the signer.
    let in_script = TxInputP2PKH::only_script(alice.into());
    let out_script = TxOutputP2PKH::only_script(bob.into());

    // Construct Protobuf payload.
    let input = SigningInput {
        // Ignored
        hash_type: 0,
        // Ignored
        amount: 0,
        // Ignored
        byte_fee: 0,
        // Ignored
        to_address: Cow::from(""),
        // Ignored
        change_address: Cow::from(""),
        private_key: vec![Cow::from(alice_privkey.as_slice())],
        scripts: Default::default(),
        utxo: vec![UnspentTransaction {
            out_point: Some(tw_proto::Bitcoin::Proto::OutPoint {
                hash: Cow::from(txid.as_slice()),
                index: 0,
                // Ignored
                sequence: 0,
            }),
            script: Cow::from(in_script.as_bytes()),
            amount: FULL_AMOUNT as i64,
            variant: TransactionVariant::P2PKH,
            spendingScript: Cow::default(),
        }],
        // Ignored
        use_max_amount: false,
        // Ignored
        coin_type: 0,
        plan: Some(TransactionPlan {
            // Ignored
            amount: 0,
            // Ignored
            available_amount: 0,
            // Ignored
            fee: 0,
            // Ignored
            change: 0,
            utxos: vec![UnspentTransaction {
                out_point: Some(tw_proto::Bitcoin::Proto::OutPoint {
                    hash: Cow::from([].as_slice()),
                    index: 0,
                    sequence: 0,
                }),
                script: Cow::from(out_script.as_bytes()),
                amount: SEND_AMOUNT as i64,
                variant: TransactionVariant::P2PKH,
                spendingScript: Cow::default(),
            }],
            // Ignored
            branch_id: Cow::from([].as_slice()),
            // Ignored
            error: tw_proto::Common::Proto::SigningError::OK,
            // Ignored
            output_op_return: Cow::from([].as_slice()),
        }),
        // Ignored
        lock_time: 0,
        // Ignored
        output_op_return: Cow::from([].as_slice()),
    };

    let sig = taproot_build_and_sign_transaction(input).unwrap();
    assert_eq!(hex::encode(&sig, false), EXPECTED_RAW_SIGNED);
}

#[test]
fn proto_sign_input_p2pkh_output_p2wpkh() {
    use crate::tests::p2wpkh::{
        ALICE_WIF, BOB_WIF, EXPECTED_RAW_SIGNED, FULL_AMOUNT, SEND_TO_BOB, TXID,
    };

    let alice: secp256k1::KeyPair = keypair_from_wif(ALICE_WIF).unwrap();
    let alice_privkey = alice.secret_bytes();
    let bob = keypair_from_wif(BOB_WIF).unwrap();

    // Note that the Txid must be reversed.
    let txid: Vec<u8> = hex::decode(TXID).unwrap().into_iter().rev().collect();

    // Prepare the scripts.
    let in_script = TxInputP2PKH::only_script(alice.into());
    let out_script = TxOutputP2WPKH::only_script(bob.try_into().unwrap());

    // Construct Protobuf payload.
    let input = SigningInput {
        hash_type: 0,
        amount: 0,
        byte_fee: 0,
        to_address: Cow::from(""),
        change_address: Cow::from(""),
        private_key: vec![Cow::from(alice_privkey.as_slice())],
        scripts: Default::default(),
        utxo: vec![UnspentTransaction {
            out_point: Some(tw_proto::Bitcoin::Proto::OutPoint {
                hash: Cow::from(txid.as_slice()),
                index: 0,
                sequence: 0,
            }),
            script: Cow::from(in_script.as_bytes()),
            amount: FULL_AMOUNT as i64,
            variant: TransactionVariant::P2PKH,
            spendingScript: Cow::default(),
        }],
        use_max_amount: false,
        coin_type: 0,
        plan: Some(TransactionPlan {
            amount: 0,
            available_amount: 0,
            fee: 0,
            change: 0,
            utxos: vec![UnspentTransaction {
                out_point: Some(tw_proto::Bitcoin::Proto::OutPoint {
                    hash: Cow::from([].as_slice()),
                    index: 0,
                    sequence: 0,
                }),
                script: Cow::from(out_script.as_bytes()),
                amount: SEND_TO_BOB as i64,
                variant: TransactionVariant::P2WPKH,
                spendingScript: Cow::default(),
            }],
            branch_id: Cow::from([].as_slice()),
            error: tw_proto::Common::Proto::SigningError::OK,
            output_op_return: Cow::from([].as_slice()),
        }),
        lock_time: 0,
        output_op_return: Cow::from([].as_slice()),
    };

    let sig = taproot_build_and_sign_transaction(input).unwrap();
    assert_eq!(hex::encode(&sig, false), EXPECTED_RAW_SIGNED);
}

#[test]
fn proto_sign_input_p2pkh_output_p2tr_key_path() {
    use crate::tests::p2tr_key_path::{
        ALICE_WIF, BOB_WIF, EXPECTED_RAW_SIGNED, FULL_AMOUNT, SEND_TO_BOB, TXID,
    };

    let alice: secp256k1::KeyPair = keypair_from_wif(ALICE_WIF).unwrap();
    let alice_privkey = alice.secret_bytes();
    let bob = keypair_from_wif(BOB_WIF).unwrap();

    // Note that the Txid must be reversed.
    let txid: Vec<u8> = hex::decode(TXID).unwrap().into_iter().rev().collect();

    // Prepare the scripts.
    let in_script = TxInputP2PKH::only_script(alice.into());
    let out_script = TxOutputP2TRKeyPath::only_script(bob.try_into().unwrap());

    // Construct Protobuf payload.
    let input = SigningInput {
        hash_type: 0,
        amount: 0,
        byte_fee: 0,
        to_address: Cow::from(""),
        change_address: Cow::from(""),
        private_key: vec![Cow::from(alice_privkey.as_slice())],
        scripts: Default::default(),
        utxo: vec![UnspentTransaction {
            out_point: Some(tw_proto::Bitcoin::Proto::OutPoint {
                hash: Cow::from(txid.as_slice()),
                index: 0,
                sequence: 0,
            }),
            script: Cow::from(in_script.as_bytes()),
            amount: FULL_AMOUNT as i64,
            variant: TransactionVariant::P2PKH,
            spendingScript: Cow::default(),
        }],
        use_max_amount: false,
        coin_type: 0,
        plan: Some(TransactionPlan {
            amount: 0,
            available_amount: 0,
            fee: 0,
            change: 0,
            utxos: vec![UnspentTransaction {
                out_point: Some(tw_proto::Bitcoin::Proto::OutPoint {
                    hash: Cow::from([].as_slice()),
                    index: 0,
                    sequence: 0,
                }),
                script: Cow::from(out_script.as_bytes()),
                amount: SEND_TO_BOB as i64,
                variant: TransactionVariant::P2TRKEYPATH,
                spendingScript: Cow::default(),
            }],
            branch_id: Cow::from([].as_slice()),
            error: tw_proto::Common::Proto::SigningError::OK,
            output_op_return: Cow::from([].as_slice()),
        }),
        lock_time: 0,
        output_op_return: Cow::from([].as_slice()),
    };

    let sig = taproot_build_and_sign_transaction(input).unwrap();
    assert_eq!(hex::encode(&sig, false), EXPECTED_RAW_SIGNED);
}

#[test]
fn proto_sign_brc20_transfer_inscription_commit() {
    use crate::tests::brc20_transfer::{
        ALICE_WIF, BOB_WIF, BRC20_AMOUNT, BRC20_INSCRIBE_AMOUNT, BRC20_TICKER, COMMIT_TXID,
        COMMIT_TX_RAW, FOR_FEE_AMOUNT, FULL_AMOUNT,
    };

    let alice = keypair_from_wif(ALICE_WIF).unwrap();
    let alice_privkey = alice.secret_bytes();
    let alice_recipient = Recipient::<PublicKey>::from(&alice);
    let bob = keypair_from_wif(BOB_WIF).unwrap();

    let ticker = Ticker::new(BRC20_TICKER.to_string())
        .unwrap()
        .to_byte_array();

    // Note that the Txid must be reversed.
    let txid: Vec<u8> = hex::decode(COMMIT_TXID)
        .unwrap()
        .into_iter()
        .rev()
        .collect();

    // Build input script.
    let pubkey = alice_recipient.public_key().to_bytes();
    let input_p2wpkh = unsafe {
        tw_build_p2wpkh_script(FULL_AMOUNT as i64, pubkey.as_ptr(), pubkey.len()).into_vec()
    };
    let input_p2wpkh: TransactionOutput = tw_proto::deserialize(&input_p2wpkh).unwrap();

    // Build inscription output.
    let output_inscribe = unsafe {
        tw_build_brc20_inscribe_transfer(
            ticker.as_ptr(),
            BRC20_AMOUNT,
            BRC20_INSCRIBE_AMOUNT as i64,
            pubkey.as_ptr(),
            pubkey.len(),
        )
        .into_vec()
    };
    let output_inscribe: TransactionOutput = tw_proto::deserialize(&output_inscribe).unwrap();

    // Build change output.
    let output_change = unsafe {
        tw_build_p2wpkh_script(FOR_FEE_AMOUNT as i64, pubkey.as_ptr(), pubkey.len()).into_vec()
    };
    let output_change: TransactionOutput = tw_proto::deserialize(&output_change).unwrap();

    // Construct Protobuf payload.
    let input = SigningInput {
        hash_type: 0,
        amount: 0,
        byte_fee: 0,
        to_address: Cow::from(""),
        change_address: Cow::from(""),
        private_key: vec![Cow::from(alice_privkey.as_slice())],
        scripts: Default::default(),
        utxo: vec![UnspentTransaction {
            out_point: Some(tw_proto::Bitcoin::Proto::OutPoint {
                hash: Cow::from(txid.as_slice()),
                index: 1,
                sequence: 0,
            }),
            script: input_p2wpkh.script,
            amount: FULL_AMOUNT as i64,
            variant: TransactionVariant::P2WPKH,
            spendingScript: Cow::default(),
        }],
        use_max_amount: false,
        coin_type: 0,
        plan: Some(TransactionPlan {
            amount: 0,
            available_amount: 0,
            fee: 0,
            change: 0,
            utxos: vec![
                UnspentTransaction {
                    out_point: Some(tw_proto::Bitcoin::Proto::OutPoint {
                        hash: Cow::from([].as_slice()),
                        index: 0,
                        sequence: 0,
                    }),
                    script: output_inscribe.script,
                    amount: BRC20_INSCRIBE_AMOUNT as i64,
                    variant: TransactionVariant::BRC20TRANSFER,
                    // Not relevant in the "commit" stage.
                    spendingScript: Cow::default(),
                },
                UnspentTransaction {
                    out_point: Some(tw_proto::Bitcoin::Proto::OutPoint {
                        hash: Cow::from([].as_slice()),
                        index: 0,
                        sequence: 0,
                    }),
                    script: output_change.script,
                    amount: FOR_FEE_AMOUNT as i64,
                    variant: TransactionVariant::P2WPKH,
                    spendingScript: Cow::default(),
                },
            ],
            branch_id: Cow::from([].as_slice()),
            error: tw_proto::Common::Proto::SigningError::OK,
            output_op_return: Cow::from([].as_slice()),
        }),
        lock_time: 0,
        output_op_return: Cow::from([].as_slice()),
    };

    let sig = taproot_build_and_sign_transaction(input).unwrap();
    assert_eq!(hex::encode(&sig, false), COMMIT_TX_RAW);
}