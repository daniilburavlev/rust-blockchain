use crate::chain::config::Config;
use crate::chain::tx;
use crate::net::client::Client;
use crate::{chain, net};
use clap::{Parser, Subcommand};
use wallet::wallet::Wallet;

#[derive(Parser)]
#[command(version, about, long_about = "xhcg-blockchain node/client")]
pub struct Cli {
    #[arg(long, value_name = "config")]
    config: Option<String>,
    #[command(subcommand)]
    chain: ChainCmd,
}

#[derive(Subcommand)]
pub enum ChainCmd {
    #[clap(about = "Create a new wallet")]
    Create,
    #[clap(about = "Start chain node")]
    Start,
    #[clap(about = "Create new transaction")]
    Tx {
        #[arg(long, value_name = "from")]
        from: String,
        #[arg(long, value_name = "to")]
        to: String,
        #[arg(long, value_name = "amount")]
        amount: String,
    },
    #[clap(about = "Stake some value")]
    Stake {
        #[arg(long, value_name = "from")]
        from: String,
        #[arg(long, value_name = "amount")]
        amount: String,
    },
}

async fn create_wallet(config: &chain::config::Config) -> Result<(), std::io::Error> {
    println!("Enter password:");
    let password = rpassword::read_password()?;
    let wallet = Wallet::new();
    wallet.write(&config.keystore_path(), password.as_bytes())?;
    println!("Wallet created, address: {}", wallet.address());
    Ok(())
}

async fn stake(
    config: &chain::config::Config,
    from: String,
    amount: String,
) -> Result<(), Box<dyn std::error::Error>> {
    new_tx(config, from, String::from("STAKE"), amount).await
}

async fn new_tx(
    config: &chain::config::Config,
    from: String,
    to: String,
    amount: String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Enter password:");
    let password = rpassword::read_password()?;
    let wallet = Wallet::read(&config.keystore_path(), from.as_str(), password.as_bytes())?;
    let mut client = Client::new(config).await?;
    let nonce = client.get_nonce(from).await;
    let tx = tx::Tx::new(&wallet, to, amount, nonce + 1)?;
    println!("Tx created: {:?}", tx);
    if client.send_tx(&tx).await {
        println!("Transaction successfully submitted");
    } else {
        println!("Transaction invalid");
    }
    Ok(())
}

async fn start_node(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let mut node = net::node::Node::new(config)?;
    if !config.nodes().is_empty() {
        node.sync(config).await?;
    }
    node.start().await?;
    Ok(())
}

pub async fn start_cli() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config = if let Some(config_path) = cli.config {
        Config::from_file(config_path.as_str())?
    } else {
        Config::from_file(chain::config::DEFAULT_CONFIG_PATH)?
    };
    match cli.chain {
        ChainCmd::Create => create_wallet(&config).await?,
        ChainCmd::Stake { from, amount } => stake(&config, from, amount).await?,
        ChainCmd::Start => start_node(&config).await?,
        ChainCmd::Tx { from, to, amount } => new_tx(&config, from, to, amount).await?,
    }
    Ok(())
}
