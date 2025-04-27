use swarmsync_core::cli::run_cli;

#[tokio::main]
async fn main() {
    if let Err(e) = run_cli().await {
        eprintln!("CLI error:\n{:#}", e);
    }
}
