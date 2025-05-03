use dialoguer::{theme::ColorfulTheme, Input, Select};
use reqwest;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Persistent configuration placed in `consumer_config.json`.
#[derive(Serialize, Deserialize, Debug)]
pub struct CoreConfig {
    pub base_url: String,
    pub last_username: Option<String>,
}

/// Absolute path of the config file (working directory).
pub fn config_file_path() -> PathBuf {
    PathBuf::from("consumer_config.json")
}

/// Ask the user where Swarm-Sync Core is running and persist the answer.
/// If a saved config exists and is reachable, offer to reuse it.
pub async fn choose_core_location() -> anyhow::Result<String> {
    // ── 1. Check existing config ──────────────────────────────────
    let cfg_path = config_file_path();

    if let Ok(raw) = fs::read_to_string(&cfg_path) {
        if let Ok(cfg) = serde_json::from_str::<CoreConfig>(&raw) {
            // Offer reuse
            let items = &[
                format!("Use saved ({})", cfg.base_url),
                "Change Core server".to_string(),
            ];
            let idx = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Previous configuration found")
                .items(items)
                .default(0)
                .interact()?;

            if idx == 0 {
                if is_reachable(&cfg.base_url).await {
                    println!("✅ Connected to Core at {}", cfg.base_url);
                    return Ok(cfg.base_url);
                }
                println!("❌ Saved Core unreachable. Reconfiguring…");
            }
        }
    }

    let opts = &["Local", "Remote"];
    let choice = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Where is the Swarm-Sync Core running?")
        .items(opts)
        .default(0)
        .interact()?;

    // Use CORE_API_URL inside Docker, fallback to localhost on host
    let default_local =
        std::env::var("CORE_API_URL").unwrap_or_else(|_| "http://127.0.0.1".to_string());

    let base_url = match choice {
        0 => default_local,
        1 => {
            let ip: String = Input::new()
                .with_prompt("Enter Core server IP")
                .interact_text()?;
            format!("http://{}", ip.trim())
        }
        _ => unreachable!(),
    };
    // ── 3. Verify connectivity, then save ─────────────────────────
    if !is_reachable(&base_url).await {
        println!("❌ Could not connect to Core at {}.", base_url);
        std::process::exit(1);
    }

    println!("✅ Core is reachable at {}", &base_url);
    let cfg = CoreConfig {
        base_url: base_url.clone(),
        last_username: None, // will be filled after successful auth
    };
    fs::write(&cfg_path, serde_json::to_string_pretty(&cfg)?)?;
    Ok(base_url)
}

/// Simple GET / to ensure the server is up.
async fn is_reachable(url: &str) -> bool {
    matches!(
        reqwest::get(format!("{}/", url))
            .await
            .map(|r| r.status().is_success()),
        Ok(true)
    )
}
