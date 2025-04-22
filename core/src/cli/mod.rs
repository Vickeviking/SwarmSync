pub mod menu;
pub mod utils;
pub mod views;

use crate::cli::menu::main_menu;

pub async fn run_cli() -> anyhow::Result<()> {
    main_menu().await
}
