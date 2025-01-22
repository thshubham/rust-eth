use std::{path::PathBuf, time::Duration};

use ethers::{
    abi::{ AbiEncode},
    contract::Contract,
    contract::ContractFactory,
    middleware::SignerMiddleware,
    providers::{Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{BlockNumber, U256},
    utils::Ganache,
};


use ethers_solc::{Artifact,ConfigurableArtifacts, Project, ProjectCompileOutput, ProjectPathsConfig};
use eyre::{eyre, Context, ContextCompat, Result};
pub type SingerDeployedContract<T> = Contract<SignerMiddleware<Provider<T>, LocalWallet>>;

#[tokio::main]
async fn main() -> Result<()> {
    let ganache = Ganache::new()
        .mnemonic("abstract vacuum mammal awkward pudding scene penalty purchase dinner depart evoke puzzle")
        .spawn();
    println!("The Ganache client is {}", ganache.endpoint());

    let wallet: LocalWallet = ganache.keys()[0].clone().into();
    let first_address = wallet.address();
    println!("Default wallet address: {}", first_address.encode_hex());

    let provider = Provider::try_from(ganache.endpoint())?.interval(Duration::from_millis(10));
    let chain_id = provider.get_chainid().await?.as_u64();
    println!("Ganache started with chain ID {}", chain_id);

    let project = compile("examples/").await?;
    print_project(project.clone()).await?;

    let balance = provider.get_balance(wallet.address(), None).await?;
    println!(
        "Wallet first address {} balance: {}",
        wallet.address().encode_hex(),
        balance
    );

    let contract_name = "BUSDImplementation";
    let contract_file_path = format!("{}/", "examples");
let contract = project
    .find(&contract_file_path, contract_name)
    .context("Contract not found")?
    .clone();

    // Get ABI and bytecode
    let (abi, bytecode, _) = contract.try_into_parts().context("Failed to extract parts from the contract")?;
    let abi = abi;
    let bytecode = bytecode;

    // Create signer client
    let wallet = wallet.with_chain_id(chain_id);
    let client = SignerMiddleware::new(provider.clone(), wallet.clone());

    // Deploy contract
    let factory = ContractFactory::new(abi.clone(), bytecode, client.clone().into());

    let mut deployer = factory.deploy(())?;
    let block = provider
        .get_block(BlockNumber::Latest)
        .await?
        .context("Failed to get block")?;
    let gas_price = block
        .base_fee_per_gas
        .context("Failed to get the base fee of the next block")?;
    deployer.tx.set_gas_price::<U256>(gas_price);
    let contract = deployer.legacy().send().await?;
    println!(
        "BUSDImplementation contract address: {}",
        contract.address().encode_hex()
    );

    Ok(())
}

pub async fn compile(root: &str) -> Result<ProjectCompileOutput<ConfigurableArtifacts>> {
    let root = PathBuf::from(root);
    if !root.exists() {
        return Err(eyre!("Project root does not exist"));
    }
    let paths = ProjectPathsConfig::builder()
        .root(&root)
        .sources(&root)
        .build()?;
    let project = Project::builder()
        .paths(paths)
        .set_auto_detect(true)
        .no_artifacts()
        .build()?;
    let output = project.compile()?;
    if output.has_compiler_errors() {
        return Err(eyre!("Compiling Solidity failed: {:?}", output.output().errors));
    }
    Ok(output)
}

pub async fn print_project(project: ProjectCompileOutput<ConfigurableArtifacts>) -> Result<()> {
    let artifacts = project.into_artifacts();
    for (id, artifact) in artifacts {
        let name = id.name;
        let abi = artifact.abi.context("Missing ABI for contract")?;
        println!("{}", "=".repeat(80));
        println!("Contract: {:?}", name);
        println!("ABI: {:?}", abi);
    }
    Ok(())
}
