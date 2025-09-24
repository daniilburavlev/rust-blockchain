mod blockchain;
mod cli;
mod net;
mod test;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cli::cli::start_cli().await
}
