//! CommandDeck CLI
//!
//! This is the main entry point for the CommandDeck CLI.
//! A TUI allowing for the user to upload and fetch jobs.

use crate::menu::main_menu;

pub mod menu;
pub mod views;

/// Tokio runtime
#[tokio::main]
async fn main() {
    // run the CLI
    if let Err(e) = run_cli().await {
        eprintln!("CLI error:\n{:#}", e);
    }
}

pub async fn run_cli() -> anyhow::Result<()> {
    // run the main menu
    main_menu().await
}
