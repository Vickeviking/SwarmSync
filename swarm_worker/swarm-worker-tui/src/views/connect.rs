use dialoguer::{theme::ColorfulTheme, Input, Select};
use swarm_worker_common::config::{self, CoreConfig};
use swarm_worker_common::net::is_reachable;

/// Ask the user where Swarm-Sync Core is running and persist the answer.
/// If a saved config exists and is reachable, offer to reuse it.
pub async fn choose_core_location() -> anyhow::Result<String> {
    // ── 1. Check existing config ──────────────────────────────────
    let cfg: CoreConfig = config::load_core_config()?;
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

    let opts = &["Local", "Remote"];
    let choice = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Where is the Swarm-Sync Core running?")
        .items(opts)
        .default(0)
        .interact()?;

    //WARNING: Add inside docker , other wise wont be found
    // Use CORE_API_URL inside Docker, fallback to localhost on host
    let default_local =
        std::env::var("CORE_API_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());

    let base_url = match choice {
        0 => default_local,
        1 => {
            let ip: String = Input::new()
                .with_prompt("Enter Core server IP")
                .interact_text()?;
            format!("http://{}:8000", ip.trim())
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
        worker_status_enum: None,
        worker_id: None,
    };
    config::save_core_config(&cfg);
    Ok(base_url)
}
