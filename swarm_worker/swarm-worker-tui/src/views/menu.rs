use crate::state;
use anyhow::{anyhow, bail, Error};
use dialoguer::{theme::ColorfulTheme, Select};
use lazy_static::lazy_static;
use swarm_worker_common::{
    commands,
    config::{self, config_file_path},
    model::WorkerStatusEnum,
    net::Session,
};

/// Main menu loop presenting user actions and handling navigation.
pub async fn main_menu() -> anyhow::Result<()> {
    loop {
        // Define the menu options
        let options = vec![
            "Start Worker",
            "Stop Worker",
            "View Worker Status",
            "User Settings",
            "Quit",
        ];
        let worker_status_string_res = view_worker_status().await;
        let worker_status_string = match worker_status_string_res {
            Ok(worker_status) => worker_status,
            _ => "Not configured".to_string(),
        };
        let choice = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(&format!("Main Menu – {}", worker_status_string))
            .items(&options)
            .default(0)
            .interact()?;

        match choice {
            0 => {
                let _ = start_worker();
                continue;
            }
            1 => {
                stop_worker();
                continue;
            }
            2 => {
                let worker_status_string_res = view_worker_status().await;
                let worker_status_string = match worker_status_string_res {
                    Ok(worker_status) => worker_status,
                    _ => "Not configured".to_string(),
                };
                println!("worker status: {}", worker_status_string);
                continue;
            }
            3 => {
                // ── User-settings submenu ────
                user_settings_tui().await?;
                continue; // back to main loop
            }
            4 => {
                // Quit the application
                println!("👋 Exiting application. Goodbye!");
                std::process::exit(0);
            }
            _ => unreachable!(),
        }
    }
}

// Assume a global or static mutable for the Child handle:
use std::sync::Mutex;
lazy_static! {
    static ref WORKER_PROCESS: Mutex<Option<std::process::Child>> = Mutex::new(None);
}

fn start_worker() -> anyhow::Result<(), anyhow::Error> {
    let mut proc_slot = WORKER_PROCESS.lock().unwrap();
    if proc_slot.is_some() {
        println!("Worker is already running!");
        return Ok(());
    }

    // Spawn the worker process
    let child = std::process::Command::new("swarm-worker")
        .arg("--config")
        .arg(&config_file_path())
        .spawn()
        .expect("Failed to spawn swarm-worker");

    println!("Swarm worker started (PID {})", child.id());

    // Store it first
    *proc_slot = Some(child);

    // Now spawn a thread that will wait on the global handle,
    // *not* on the local `child` you just moved into proc_slot.
    std::thread::spawn(|| {
        let mut guard = WORKER_PROCESS.lock().unwrap();
        if let Some(mut child) = guard.take() {
            if let Ok(status) = child.wait() {
                println!(
                    "Swarm worker process {} exited with status {}",
                    child.id(),
                    status
                );
            }
        }
    });

    Ok(())
}

fn status_worker() -> String {
    let mut proc_slot = WORKER_PROCESS.lock().unwrap();
    if let Some(child) = proc_slot.as_mut() {
        match child.try_wait() {
            Ok(Some(status)) => {
                // Process has exited
                proc_slot.take(); // set to None
                format!("Worker stopped (exit code {})", status.code().unwrap_or(-1))
            }
            Ok(None) => {
                format!("Worker is running (PID {})", child.id())
            }
            Err(e) => {
                format!("Error checking worker status: {}", e)
            }
        }
    } else {
        "Worker is not running.".to_string()
    }
}

fn stop_worker() {
    let mut proc_slot = WORKER_PROCESS.lock().unwrap();
    if let Some(mut child) = proc_slot.take() {
        println!("Stopping worker (PID {})...", child.id());
        if let Err(e) = child.kill() {
            println!("Failed to kill worker: {}", e);
        } else {
            let _ = child.wait(); // ensure it's reaped
            println!("Worker stopped.");
        }
    } else {
        println!("No worker running.");
    }
}

pub async fn view_worker_status() -> anyhow::Result<String, Error> {
    let session: Session = state::get_session();
    let cfg = config::load_core_config()?;
    let worker_id: i32 = cfg.worker_id.ok_or_else(|| anyhow!("worker id not set"))?;
    let worker_status: WorkerStatusEnum = commands::get_worker_status(&session, worker_id).await?;
    return Ok(format!("Worker status: [{}]", worker_status.to_string()).to_string());
}

async fn user_settings_tui() -> anyhow::Result<(), anyhow::Error> {
    let sub_opts = vec!["Change Core / IP", "Update Profile", "Back"];
    let sel = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("User Settings")
        .items(&sub_opts)
        .default(0)
        .interact()?;

    match sel {
        0 => {
            // re-run core+auth flow (same as before)
            let new_base = crate::views::connect::choose_core_location().await?;
            let new_session = crate::views::auth::auth_flow(&new_base).await?;
            state::set_session(new_session)?;
            println!("✅ Configuration updated.");
            Ok(())
        }
        1 => {
            // update-profile flow
            let sess = state::get_session();
            let new_username: String = dialoguer::Input::new()
                .with_prompt("New username (leave blank to keep)")
                .allow_empty(true)
                .with_initial_text(&sess.user.username)
                .interact_text()?;
            let new_email: String = dialoguer::Input::new()
                .with_prompt("New email (leave blank to keep)")
                .allow_empty(true)
                .with_initial_text(&sess.user.email)
                .interact_text()?;
            let new_password: String = dialoguer::Input::new()
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
                    // refresh session.user
                    let mut s = sess.clone();
                    s.user = updated;
                    state::set_session(s)?;
                    println!("✅ Profile updated.");
                    Ok(())
                }
                Err(err) => Err(err),
            }
        }
        _ => bail!("Not a valid option"),
    }
}
