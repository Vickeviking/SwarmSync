use swarmsync_core::cli::run_cli;
use tokio;

#[tokio::main]
async fn main() {
    if let Err(e) = run_cli().await {
        eprintln!("CLI error: {}", e);
    }
}
