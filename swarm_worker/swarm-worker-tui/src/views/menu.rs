use crate::state;
use anyhow::{anyhow, Context, Result};
use dialoguer::{theme::ColorfulTheme, Input, Select};
use lazy_static::lazy_static;
use reqwest;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use swarm_worker_common::{
    commands,
    config::{self, save_core_config},
    ipc::{SHUTDOWN_SOCKET, WORKER_LOG},
    model::WorkerStatusEnum,
    net::Session,
};

lazy_static! {
    static ref WORKER_PROCESS: Mutex<Option<Child>> = Mutex::new(None);
}

// ═══════════════════════════════════════════════════════════════════════════════════════════
// Helper: is the worker configured?
// ═══════════════════════════════════════════════════════════════════════════════════════════
fn worker_id_set() -> bool {
    config::load_core_config()
        .ok()
        .and_then(|c| c.worker_id)
        .is_some()
}

// ═══════════════════════════════════════════════════════════════════════════════════════════
// Configure‑worker flow (async)
// ═══════════════════════════════════════════════════════════════════════════════════════════
async fn configure_worker() -> Result<()> {
    // Attempt async fetch of public IP
    let public_ip = match reqwest::get("https://api.ipify.org?format=text").await {
        Ok(resp) => resp.text().await.ok(),
        Err(_) => None,
    };

    if let Some(ip) = public_ip {
        println!("Public IP address: {ip}");
    } else {
        println!("Could not determine public IP automatically – please check manually.");
    }

    println!("Register this worker in Core → CommandDeck.");
    println!("Locate your new worker ID under CommandDeck → Worker Management.\n");

    // Prompt for ID – leave blank to abort
    let input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter new worker ID (leave blank to cancel)")
        .allow_empty(true)
        .interact_text()?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        println!("Configuration cancelled – worker ID unchanged.");
        return Ok(());
    }

    // Parse ID
    let id: i32 = trimmed
        .parse()
        .map_err(|_| anyhow!("Invalid worker ID – must be an integer"))?;

    // Persist the new worker_id
    let mut cfg = config::load_core_config()
        .context("Failed to load existing config – cannot update worker ID")?;
    cfg.worker_id = Some(id);
    save_core_config(&cfg).context("Failed to save updated worker ID to config file")?;

    println!("✅ Worker ID {id} saved. You can now start the worker.");
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════════════════
// TUI entry‑point
// ═══════════════════════════════════════════════════════════════════════════════════════════

pub async fn main_menu() -> Result<()> {
    loop {
        let opts = [
            "Start Worker",
            "Stop Worker",
            "View Worker Status",
            "View Worker Log",
            "User Settings",
            "Configure Worker",
            "Quit",
        ];

        let status_label = if worker_id_set() {
            view_worker_status()
                .await
                .unwrap_or_else(|_| "Status unknown".into())
        } else {
            "Worker ID not set".into()
        };

        let choice = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(&format!("Main Menu – {}", status_label))
            .items(&opts)
            .default(0)
            .interact()?;

        match choice {
            0 => {
                if worker_id_set() {
                    start_worker()?;
                } else {
                    println!("Worker is not configured – please choose 'Configure Worker' first.");
                }
            }
            1 => {
                if worker_id_set() {
                    stop_worker()?;
                } else {
                    println!("Worker is not configured – nothing to stop.");
                }
            }
            2 => {
                if worker_id_set() {
                    println!(
                        "{}",
                        view_worker_status()
                            .await
                            .unwrap_or_else(|e| format!("Error: {e}"))
                    );
                } else {
                    println!("Worker is not configured.");
                }
            }
            3 => display_worker_log()?,
            4 => user_settings_tui().await?,
            5 => configure_worker().await?,
            6 => {
                println!("👋 Exiting application. Goodbye!");
                std::process::exit(0);
            }
            _ => unreachable!(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════════
// Process management (unchanged)
// ═══════════════════════════════════════════════════════════════════════════════════════════

fn start_worker() -> Result<()> {
    let mut guard = WORKER_PROCESS.lock().unwrap();
    if guard.is_some() {
        println!("Worker is already running!");
        return Ok(());
    }

    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(WORKER_LOG)
        .map_err(|e| anyhow!("Cannot open log file {WORKER_LOG}: {e}"))?;
    let log_clone = log_file.try_clone()?;

    let child = Command::new("swarm-worker")
        .arg("--config")
        .arg(&config::config_file_path())
        .stdin(Stdio::null())
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::from(log_clone))
        .spawn()
        .map_err(|e| anyhow!("Failed to spawn swarm-worker: {e}"))?;

    println!("Swarm worker started (PID {})", child.id());
    *guard = Some(child);

    thread::spawn(|| loop {
        let still_running = {
            let mut g = WORKER_PROCESS.lock().unwrap();
            if let Some(child) = g.as_mut() {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        println!("Swarm worker {} exited with status {}", child.id(), status);
                        g.take();
                        false
                    }
                    Ok(None) => true,
                    Err(e) => {
                        println!("Error polling worker: {e}");
                        false
                    }
                }
            } else {
                false
            }
        };
        if !still_running {
            break;
        }
        thread::sleep(Duration::from_secs(1));
    });

    Ok(())
}

fn stop_worker() -> Result<()> {
    let mut child_opt = {
        let mut guard = WORKER_PROCESS.lock().unwrap();
        guard.take()
    };

    if child_opt.is_none() {
        println!("No worker running.");
        return Ok(());
    }

    println!("Sending graceful shutdown signal …");
    match UnixStream::connect(SHUTDOWN_SOCKET) {
        Ok(mut stream) => {
            let _ = stream.write_all(b"shutdown");
        }
        Err(e) => println!("⚠️  Could not connect to shutdown socket: {e}. Falling back to SIGKILL"),
    }

    thread::sleep(Duration::from_secs(2));

    if let Some(mut child) = child_opt.take() {
        match child.try_wait() {
            Ok(Some(_)) => println!("Worker shut down gracefully."),
            Ok(None) => {
                println!("Graceful shutdown timed out – killing {} …", child.id());
                child.kill().ok();
                child.wait().ok();
                println!("Worker stopped.");
            }
            Err(e) => println!("Error while waiting for worker: {e}"),
        }
    }

    let _ = fs::remove_file(SHUTDOWN_SOCKET);
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════════════════
// Log viewing helper
// ═══════════════════════════════════════════════════════════════════════════════════════════

fn display_worker_log() -> Result<()> {
    let contents = match fs::read_to_string(WORKER_LOG) {
        Ok(s) => s,
        Err(e) => {
            println!("Cannot read log file {WORKER_LOG}: {e}");
            return Ok(());
        }
    };

    if contents.is_empty() {
        println!("(log file is empty)");
        return Ok(());
    }

    const MAX_LINES: usize = 200;
    let lines: Vec<_> = contents.lines().collect();
    let start = lines.len().saturating_sub(MAX_LINES);
    for l in &lines[start..] {
        println!("{l}");
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════════════════
// Remote status helpers
// ═══════════════════════════════════════════════════════════════════════════════════════════

pub async fn view_worker_status() -> Result<String> {
    let session: Session = state::get_session();
    let cfg = config::load_core_config()?;
    let worker_id = cfg.worker_id.ok_or_else(|| anyhow!("worker id not set"))?;
    let status: WorkerStatusEnum = commands::get_worker_status(&session, worker_id).await?;
    Ok(format!("Worker status: [{}]", status))
}

// ═══════════════════════════════════════════════════════════════════════════════════════════
// User settings
// ═══════════════════════════════════════════════════════════════════════════════════════════

async fn user_settings_tui() -> Result<()> {
    let opts = ["Change Core / IP", "Update Profile", "Back"];
    let sel = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("User Settings")
        .items(&opts)
        .default(0)
        .interact()?;

    match sel {
        0 => {
            let base = crate::views::connect::choose_core_location().await?;
            let new_sess = crate::views::auth::auth_flow(&base).await?;
            state::set_session(new_sess)?;
            println!("✅ Configuration updated.");
        }
        1 => {
            let sess = state::get_session();
            let new_username: String = Input::new()
                .with_prompt("New username (leave blank to keep)")
                .allow_empty(true)
                .with_initial_text(&sess.user.username)
                .interact_text()?;
            let new_email: String = Input::new()
                .with_prompt("New email (leave blank to keep)")
                .allow_empty(true)
                .with_initial_text(&sess.user.email)
                .interact_text()?;
            let new_password: String = Input::new()
                .with_prompt("New password (optional)")
                .allow_empty(true)
                .interact_text()?;

            let result = commands::update_user(
                &sess,
                if new_username.is_empty() {
                    &sess.user.username
                } else {
                    &new_username
                },
                if new_email.is_empty() {
                    &sess.user.email
                } else {
                    &new_email
                },
                if new_password.is_empty() {
                    None
                } else {
                    Some(&new_password)
                },
            )
            .await;

            match result {
                Ok(updated) => {
                    let mut s = sess.clone();
                    s.user = updated;
                    state::set_session(s)?;
                    println!("✅ Profile updated.");
                }
                Err(e) => println!("Error updating profile: {e}"),
            }
        }
        _ => {}
    }

    Ok(())
}
