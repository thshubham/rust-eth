use std::time::Duration;

use ethers::{abi::{AbiEncode, Address}, providers::{Middleware, Provider}, signers::{LocalWallet, Signer}, types::{ TransactionRequest, U256}, utils::Ganache};
use eyre::{ContextCompat, Result};

#[tokio::main]
async fn main()-> Result<()> {

let ganache = Ganache::new()
    .mnemonic("abstract vacuum mammal awkward pudding scene penalty purchase dinner depart evoke puzzle")
    .spawn();
println!("the ganache client is {}",ganache.endpoint() );
let wallet:LocalWallet = ganache.keys()[0].clone().into();
let wallet_address:String = wallet.address().encode_hex();
println!("Default wallet address: {}", wallet_address);
let first_address =wallet.address();
let provider = Provider::try_from(ganache.endpoint())?.interval(Duration::from_millis(10));
let first_balance = provider.get_balance(first_address, None).await?;
println!("wallet first address balance {}", first_balance);
let other_address_hex = "0xaf206dCE72A0ef76643dfeDa34DB764E2126E646";
let other_address = "0xaf206dCE72A0ef76643dfeDa34DB764E2126E646".parse::<Address>()?;
let other_balance = provider.get_balance(other_address, None).await?;
println!(
    "Balance for address {}: {}",
    other_address_hex, other_balance
);
let tx = TransactionRequest::pay(other_address, U256::from(1000u64)).from(first_address);
let receipt = provider.send_transaction(tx, None).await?.log_msg("pening transfer").confirmations(1).await?.context("Missing receipt");
if let Some(block_number) = receipt.unwrap().block_number {
    println!("TX mined in block {}", block_number);
} else {
    eprintln!("Block number is missing in the receipt");
}
println!(
    "Balance of {} {}",
    other_address_hex,
    provider.get_balance(other_address, None).await?
);
Ok(())
}