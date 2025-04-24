mod client;
mod models;
mod state;
mod views;

use crate::views::{auth, connect, menu};
use tokio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let base_url = connect::choose_core_location().await?;
    let session = auth::auth_flow(&base_url).await?;
    state::set_session(session)?;
    menu::main_menu().await?;
    Ok(())
}
