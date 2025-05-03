use crate::menu::main_menu;

pub mod menu;
pub mod views;

#[tokio::main]
async fn main() {
    if let Err(e) = run_cli().await {
        eprintln!("CLI error:\n{:#}", e);
    }
}

pub async fn run_cli() -> anyhow::Result<()> {
    main_menu().await
}
