use bitcoincore_rpc::{ Auth, Client, RpcApi };
use bitcoincore_rpc::bitcoin::{ Amount, Network, Txid };
use bitcoincore_rpc::json::{ CreateRawTransactionInput, SignRawTransactionResult };
use std::collections::HashMap;
use hex;
// use serde_json::json;

fn main() {
    // Connect to Bitcoin Core RPC
    let rpc = Client::new(
        "http://localhost:18443",
        Auth::UserPass("alice".to_string(), "password".to_string())
    ).unwrap();

    // Create a new wallet
    let wallet_name = "testwallet_p2pool";
    let create_wallet_result = rpc.call::<serde_json::Value>("createwallet", &[wallet_name.into()]);

    // Check if the wallet creation was successful or already exists
    match create_wallet_result {
        Ok(_) => println!("Wallet '{}' created successfully!", wallet_name),
        Err(e) => println!("Wallet creation failed: {}", e),
    }

    // Connect to the new wallet
    let wallet_rpc = Client::new(
        &format!("http://localhost:18443/wallet/{}", wallet_name),
        Auth::UserPass("alice".to_string(), "password".to_string())
    ).unwrap();

    // Get a new address from the wallet
    let first_address = wallet_rpc.get_new_address(None, None).unwrap();
    println!("Generated new address: {:?}", first_address);

    let checked_address = first_address.clone();

    // Generate blocks to the new address
    rpc.generate_to_address(
        101,
        &checked_address.clone().require_network(Network::Regtest).unwrap()
    ).unwrap();

    // List unspent transactions
    let unspent = rpc.list_unspent(None, None, None, None, None).unwrap();
    let first_utxo = &unspent[0];
    let txid: Txid = first_utxo.txid;
    let vout = first_utxo.vout;

    // Create raw transaction inputs
    let input = CreateRawTransactionInput {
        txid: txid, // Dereferenced txid
        vout,
        sequence: None,
    };

    // Prepare outputs with a HashMap
    let mut outputs = HashMap::new();
    outputs.insert(
        checked_address.clone().require_network(Network::Regtest).unwrap().to_string(),
        Amount::from_btc(10.0).unwrap()
    );

    // Create raw transaction
    let first_raw_tx = rpc.create_raw_transaction(&[input], &outputs, None, None).unwrap();

    // 💰 Fund raw transaction with fee_rate = 0
    let fund_options = bitcoincore_rpc::json::FundRawTransactionOptions {
        fee_rate: Some(Amount::from_btc(0.1).unwrap()),
        ..Default::default()
    };

    let funded_result = rpc
        .fund_raw_transaction(&first_raw_tx, Some(&fund_options), Some(false))
        .unwrap();

    let funded_raw_tx = funded_result.hex;

    // Sign raw transaction
    let SignRawTransactionResult {
        hex: first_signed_tx_hex,
        complete,
        ..
    } = rpc.sign_raw_transaction_with_wallet(&funded_raw_tx, None, None).unwrap();

    assert!(complete, "Transaction signing failed");

    println!("First Signed Transaction Hex: {}", hex::encode(first_signed_tx_hex.clone()));

    // Broadcast the signed transaction
    let first_txid = rpc.send_raw_transaction(&first_signed_tx_hex).unwrap();
    println!("First Transaction ID: {}", first_txid);

    // Generate a block to confirm the transaction
    rpc.generate_to_address(
        1,
        &checked_address.clone().require_network(Network::Regtest).unwrap()
    ).unwrap();

    // Prepare a second transaction
    let second_address = rpc
        .get_new_address(None, None)
        .unwrap()
        .require_network(Network::Regtest)
        .unwrap();
    let second_checked_address = second_address.clone();

    let second_input = CreateRawTransactionInput {
        txid: first_txid,
        vout: 0,
        sequence: None,
    };

    let mut second_outputs = HashMap::new();
    second_outputs.insert(second_checked_address.to_string(), Amount::from_btc(5.0).unwrap());

    // Create second raw transaction
    let second_raw_tx = rpc.create_raw_transaction(&[second_input], &second_outputs, None, None).unwrap();

    // 💰 Fund raw transaction with fee_rate = 0
    let fund_options = bitcoincore_rpc::json::FundRawTransactionOptions {
        fee_rate: Some(Amount::from_btc(0.1).unwrap()),
        ..Default::default()
    };

    let funded_result = rpc
        .fund_raw_transaction(&second_raw_tx, Some(&fund_options), Some(false))
        .unwrap();

    let funded_raw_tx = funded_result.hex;

    // Sign raw transaction
    let SignRawTransactionResult {
        hex: second_signed_tx_hex,
        complete,
        ..
    } = rpc.sign_raw_transaction_with_wallet(&funded_raw_tx, None, None).unwrap();

    assert!(complete, "Transaction signing failed");

    println!("second Signed Transaction Hex: {}", hex::encode(second_signed_tx_hex.clone()));

    let second_txid = rpc.send_raw_transaction(&second_signed_tx_hex).unwrap();
    println!("Second Transaction ID: {}", second_txid);

    // Generate a block to confirm the second transaction
    rpc.generate_to_address(
        101,
        &checked_address.require_network(Network::Regtest).unwrap()
    ).unwrap();

    println!("Both transactions successfully completed and mined!");
}
