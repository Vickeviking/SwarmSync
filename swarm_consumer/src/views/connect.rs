use std::fs;
use std::path::PathBuf;

use dialoguer::{theme::ColorfulTheme, Input, Select};
use reqwest;
use serde::{Deserialize, Serialize};

/// Persistent configuration placed in `consumer_config.json`.
///
/// This is serialized as JSON and persisted to disk.
///
/// # Examples
/// ```
/// let cfg = CoreConfig {
///     base_url: "http://127.0.0.1:8000".to_string(),
///     last_username: None,
/// };
/// let serialized = serde_json::to_string_pretty(&cfg).unwrap();
/// let deserialized: CoreConfig = serde_json::from_str(&serialized).unwrap();
/// assert_eq!(cfg, deserialized);
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub struct CoreConfig {
    /// The base URL of the Core instance, no port, e.g. `http://127.0.0.1`
    pub base_url: String,
    /// The last username used for login. None if no username has been used.
    pub last_username: Option<String>,
}

/// Absolute path of the config file where CoreConfig is persisted.
/// # Returns
/// * `PathBuf` - The absolute path
/// # Panics
/// * Does not panic
/// # Example
/// ```
/// let path = config_file_path();
/// assert_eq!(path, PathBuf::from("consumer_config.json"));
/// ```
pub fn config_file_path() -> PathBuf {
    PathBuf::from("consumer_config.json")
}

/// Ask the user where Swarm-Sync Core is running and persist the answer.
/// If a saved config exists and is reachable, offer to reuse it.
/// If local, use docker CORE_API_HOST env var, otherwise defaults to 127.0.0.1
/// # Returns
/// * `String` - The base URL of the Core instance | NO PORT , e.g. `http://127.0.0.1`
/// * `anyhow::Error`
/// # Panics
/// * Does not panic
/// # Example
/// ```
/// let base_url = choose_core_location().await.unwrap();
/// assert_eq!(base_url, "http://127.0.0.1");
/// ```
pub async fn choose_core_location() -> anyhow::Result<String> {
    // Check for saved config
    let cfg_path = config_file_path();

    // If saved config exists, offer reuse
    if let Ok(raw) = fs::read_to_string(&cfg_path) {
        //Deserialize CoreConfig
        if let Ok(cfg) = serde_json::from_str::<CoreConfig>(&raw) {
            // Offer reuse menu
            let items = &[
                format!("Use saved ({})", cfg.base_url),
                "Change Core server".to_string(),
            ];
            // select items
            let idx = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Previous configuration found")
                .items(items)
                .default(0)
                .interact()?;

            // If use saved, try and contact it at basic unprotected hello route
            if idx == 0 {
                if is_reachable(&cfg.base_url).await {
                    println!("✅ Connected to Core at {}", cfg.base_url);
                    return Ok(cfg.base_url);
                }
                println!("❌ Saved Core unreachable. Reconfiguring…");
            }
        }
    }

    // Could not read in a saved config, or a change of config was requested
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
    // Ensure Core is reachable
    if !is_reachable(&base_url).await {
        println!("❌ Could not connect to Core at {}.", base_url);
        std::process::exit(1);
    }

    // Save CoreConfig to structure
    println!("✅ Core is reachable at {}", &base_url);
    let cfg = CoreConfig {
        base_url: base_url.clone(),
        last_username: None, // will be filled after successful auth
    };
    fs::write(&cfg_path, serde_json::to_string_pretty(&cfg)?)?;
    Ok(base_url)
}

/// Simple GET / to ensure the server is up.
/// # Returns
/// * `bool` - True if server is reachable
/// # Panics
/// * Does not panic
async fn is_reachable(url: &str) -> bool {
    matches!(
        reqwest::get(format!("{}/", url))
            .await
            .map(|r| r.status().is_success()),
        Ok(true)
    )
}
