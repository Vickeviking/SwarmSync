use crate::commands;

use super::model::WorkerStatusEnum;
use super::net::Session;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

/// Persistent configuration placed in `worker_config.json`.
#[derive(Serialize, Deserialize, Debug)]
pub struct CoreConfig {
    pub base_url: String,
    pub last_username: Option<String>,
    pub worker_status_enum: Option<WorkerStatusEnum>,
    pub worker_id: Option<i32>,
}

/// Absolute path of the config file (working directory).
pub fn config_file_path() -> PathBuf {
    // 1) ENV override (container will always have this set)
    if let Ok(path) = env::var("WORKER_CONFIG_PATH") {
        return PathBuf::from(path);
    }

    // 2) Fallback for local dev:
    //    CARGO_MANIFEST_DIR points to the current crate (swarm-worker-tui)
    //    so we go up two levels to `swarm_worker/` then into `swarm-worker-common`.
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // .../swarm-worker/swarm-worker-tui
    p.pop(); // .../swarm_worker
    p.push("swarm-worker-common");
    p.push("worker_config.json");
    p
}

/// Load existing config store in worker_config.json
///
/// # Returns
/// * Core config - Deserialized into a rust struct
pub fn load_core_config() -> Result<CoreConfig> {
    let cfg_path = config_file_path();

    let raw = match fs::read_to_string(&cfg_path) {
        Ok(contents) => contents,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Err(e).with_context(|| format!("Could not find config file {:?}", cfg_path))
        }
        Err(e) => {
            return Err(e).with_context(|| format!("failed to read config file {:?}", cfg_path))
        }
    };

    // Parse JSON with context on error
    let cfg = serde_json::from_str::<CoreConfig>(&raw)
        .with_context(|| format!("failed to parse JSON in {:?}", cfg_path))?;

    Ok(cfg)
}

/// Save core config to worker_config.json
pub fn save_core_config(cfg: &CoreConfig) -> Result<()> {
    let cfg_path = config_file_path();

    // Serialize to pretty JSON
    let json = serde_json::to_string_pretty(cfg)
        .with_context(|| format!("failed to serialize config for {:?}", cfg_path))?;

    // Write the file (overwrites any existing)
    fs::write(&cfg_path, json)
        .with_context(|| format!("failed to write config file {:?}", cfg_path))?;

    Ok(())
}

/// Retrieve worker status from worker_id through http endpoint in server
pub async fn retrieve_worker_status(session: Session) -> Result<WorkerStatusEnum> {
    let cfg = load_core_config().context("failed to load core configuration")?;

    let worker_id = cfg
        .worker_id
        .ok_or_else(|| anyhow!("no worker_id found in configuration"))?;

    let status = commands::get_worker_status(&session, worker_id)
        .await
        .context("failed to retrieve worker status from Core")?;

    Ok(status)
}
