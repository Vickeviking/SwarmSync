use crate::commands;

use super::model::WorkerStatusEnum;
use super::net::Session;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

/// Persistent configuration placed in `worker_config.json`.
#[derive(Serialize, Deserialize, Debug)]
pub struct CoreConfig {
    pub base_url: String,
    pub last_username: Option<String>,
    pub worker_status_enum: WorkerStatusEnum,
    pub worker_id: i32,
}

/// Absolute path of the config file (working directory).
pub fn config_file_path() -> PathBuf {
    PathBuf::from("worker_config.json")
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
    let cfg = load_core_config()?;
    let worker_id = cfg.worker_id;
    let worker_status: WorkerStatusEnum = commands::get_worker_status(&session, worker_id)
        .await
        .context("failed to retrieve worker status in commands")?;

    Ok(worker_status)
}
