// swarm-worker-tui/src/main.rs
#![warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![forbid(unsafe_code)]

mod state;
mod views;

use crate::views::{auth, connect, menu};
use anyhow::{Context, Result};
use std::backtrace::{Backtrace, BacktraceStatus};

#[tokio::main]
async fn main() {
    // 1️⃣  Panic hook so even panics look similar
    std::panic::set_hook(Box::new(|info| {
        eprintln!("panic: {info}");
        print_backtrace(&Backtrace::force_capture());
    }));

    // 2️⃣  Run app and pretty-print any anyhow::Error
    if let Err(err) = run().await {
        eprintln!("{:#}", err); // full context chain
        print_backtrace(err.backtrace()); // pretty back-trace
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let base_url = connect::choose_core_location()
        .await
        .context("choosing core location")?;

    let session = auth::auth_flow(&base_url)
        .await
        .context("authentication flow failed")?;

    state::set_session(session).context("storing session in global state")?;

    menu::main_menu()
        .await
        .context("main menu exited with error")?;

    Ok(())
}

/// Pretty-print a back-trace only if it was actually captured.
fn print_backtrace(bt: &Backtrace) {
    if bt.status() == BacktraceStatus::Captured {
        eprintln!("\nBacktrace:\n{}", bt); // NOTE: `{}` = Display, NOT `{:?}`
    }
}
