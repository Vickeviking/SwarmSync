#![warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![forbid(unsafe_code)]

mod client;
mod commands;
mod models;
mod state;
mod views;

use crate::views::{auth, connect, menu};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Determine backend core location (local or remote) and get base URL
    let base_url = connect::choose_core_location().await?;
    // Perform login or registration to obtain an authenticated session
    let session = auth::auth_flow(&base_url).await?;
    // Store the session (with client and user info) in global state for use in the TUI
    state::set_session(session)?;
    // Enter the main interactive menu loop
    menu::main_menu().await?;
    Ok(())
}
