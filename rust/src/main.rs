use bitcoincore_rpc::{ Auth, Client, RpcApi };
use bitcoincore_rpc::bitcoin::{ Amount, Network, Txid };
use bitcoincore_rpc::json::{
    CreateRawTransactionInput,
    SignRawTransactionResult,
    FundRawTransactionOptions,
};
use std::collections::HashMap;
use std::error::Error;

// Create a connection to the Bitcoin node
fn create_rpc_client(url: &str, username: &str, password: &str) -> Result<Client, Box<dyn Error>> {
    let rpc = Client::new(url, Auth::UserPass(username.to_string(), password.to_string()))?;
    Ok(rpc)
}

// Create a new wallet or use existing one
fn setup_wallet(rpc: &Client, wallet_name: &str) -> Result<Client, Box<dyn Error>> {
    // Try to create wallet (may fail if already exists)
    match rpc.call::<serde_json::Value>("createwallet", &[wallet_name.into()]) {
        Ok(_) => println!("Wallet '{}' created successfully!", wallet_name),
        Err(e) => println!("Wallet creation note: {}", e),
    }

    // Connect to the wallet
    let wallet_rpc = Client::new(
        &format!("http://localhost:18443/wallet/{}", wallet_name),
        Auth::UserPass("vasu".to_string(), "password".to_string())
    )?;

    Ok(wallet_rpc)
}

// Generate blocks to an address
fn generate_blocks(
    rpc: &Client,
    address: &bitcoincore_rpc::bitcoin::Address,
    count: u64
) -> Result<Vec<bitcoincore_rpc::bitcoin::BlockHash>, Box<dyn Error>> {
    let network_address = address.clone();
    if network_address.network != Network::Regtest {
        return Err("Address is not for regtest network".into());
    }

    let block_hashes = rpc.generate_to_address(count, &network_address)?;
    Ok(block_hashes)
}

// Create, fund, sign and broadcast a transaction
fn create_and_send_transaction(
    rpc: &Client,
    input: CreateRawTransactionInput,
    output_address: &bitcoincore_rpc::bitcoin::Address,
    amount: f64,
    fee_rate: f64
) -> Result<Txid, Box<dyn Error>> {
    // Prepare outputs with a HashMap
    let mut outputs = HashMap::new();
    if output_address.network != Network::Regtest {
        return Err("Output address is not for regtest network".into());
    }

    outputs.insert(output_address.clone().to_string(), Amount::from_btc(amount)?);

    // Create raw transaction
    let raw_tx = rpc.create_raw_transaction(&[input], &outputs, None, None)?;
    println!("Raw transaction created");

    // Fund raw transaction
    let fund_options = FundRawTransactionOptions {
        fee_rate: Some(Amount::from_btc(fee_rate)?),
        ..Default::default()
    };

    println!("Funding raw transaction");

    let funded_result = rpc.fund_raw_transaction(&raw_tx, Some(&fund_options), Some(false))?;
    let funded_raw_tx = funded_result.hex;

    // Sign raw transaction
    let SignRawTransactionResult {
        hex: signed_tx_hex,
        complete,
        ..
    } = rpc.sign_raw_transaction_with_wallet(&funded_raw_tx, None, None)?;

    if !complete {
        return Err("Transaction signing failed".into());
    }
    else{
        println!("Transaction signed successfully");
    }

    // Broadcast the signed transaction
    let txid = rpc.send_raw_transaction(&signed_tx_hex)?;
    println!("Transaction broadcasted");

    Ok(txid)
}

fn main() -> Result<(), Box<dyn Error>> {
    // Connect to Bitcoin Core RPC
    let rpc = create_rpc_client("http://localhost:18443", "vasu", "password")?;

    // Setup wallet
    let wallet_name = "testwallet_p2pool";
    let wallet_rpc = setup_wallet(&rpc, wallet_name)?;

    // Get a new address from the wallet
    let first_address = wallet_rpc.get_new_address(None, None)?;
    println!(
        "Generated new address: {}",
        &format!("{:?}", first_address)[26..].trim_end_matches(')')
    );

    //generating 101 blocks to get the coinbase transaction for first address
    let first_address = wallet_rpc.get_new_address(None, None)?.require_network(Network::Regtest)?;
    generate_blocks(&rpc, &first_address, 101)?;
    println!("Generated 101 blocks to get Coinbase Transaction");

    // Get wallet balance
    let balance = wallet_rpc.get_balance(None, None)?;
    println!("Wallet balance: {} BTC", balance);

    // List unspent transactions to find an input for our first transaction
    let unspent = rpc.list_unspent(None, None, None, None, None)?;
    if unspent.is_empty() {
        return Err("No unspent outputs found".into());
    }

    let first_utxo = &unspent[0];
    let input = CreateRawTransactionInput {
        txid: first_utxo.txid,
        vout: first_utxo.vout,
        sequence: None,
    };

    // Create and send first transaction
    println!("Using coinbase output for first transaction");
    
    let first_txid = create_and_send_transaction(&rpc, input, &first_address, 10.0, 0.1)?;

    // Generate a block to confirm the first transaction
    generate_blocks(&rpc, &first_address, 1)?;
    println!("First transaction ID: {}", first_txid);

    // Prepare and send second transaction using the output from the first
    let second_address = rpc.get_new_address(None, None)?.require_network(Network::Regtest)?;

    println!("Using first transaction output for second transaction");

    let second_input = CreateRawTransactionInput {
        txid: first_txid,
        vout: 0,
        sequence: None,
    };

    // Create and send second transaction
    let second_txid = create_and_send_transaction(&rpc, second_input, &second_address, 5.0, 0.1)?;

    // Generate blocks to confirm the second transaction
    generate_blocks(&rpc, &first_address, 101)?;
    println!("Second transaction ID: {}", second_txid);

    println!("All steps of competency tests successfully completed!");

    Ok(())
}
