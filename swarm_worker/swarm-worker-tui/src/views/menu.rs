use crate::{commands, state};
use anyhow::Error;
use dialoguer::{theme::ColorfulTheme, Select};
use swarm_worker_common::{commands, config::{self, save_core_config}};


lazy_static::lazy_static! {
    // store the Child handle so we can kill it later
    static ref WORKER_HANDLE: Mutex<Option<Child>> = Mutex::new(None);
}

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
        let worker_status_string = view_worker_status();
        let choice = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(fmt!("Main Menu - {}", worker_status_string))
            .items(&options)
            .default(0)
            .interact()?;

        match choice {
            0 => {
                start_worker();
                continue;
            1 => {
                stop_worker();
                continue;
            }
            2 => {
                view_worker_status();
                continue;
            }
            3 => {
                // ── User-settings submenu ────                
                user_settings_tui();
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

fn start_worker() -> Result<()> {
    // 1. Ensure Worker is not already running
    if is_worker_running() {
        println!("Worker is already running.");
        return Ok(());
    }
    // 2. Save current config (ensure auth token and base_url are up-to-date in worker_config.json)
    save_core_config(current_config)?;
    // 3. Spawn the Worker process
    let mut cmd = Command::new("swarm-worker");
    cmd.stdout(Stdio::piped())  // capture stdout to catch the IP message
       .stderr(Stdio::piped()); // (could also redirect stderr to file if needed)
    #[cfg(target_family = "unix")]
    {
        use std::os::unix::process::CommandExt;
        cmd.before_exec(|| { unsafe { libc::setsid() }; Ok(()) });
    }
    let mut child = cmd.spawn()?;
    // 4. Optionally, read one-time startup messages (like register IP) from stdout
    if let Some(stdout) = child.stdout.take() {
        use std::io::{BufReader, BufRead};
        let reader = BufReader::new(stdout);
        if let Some(Ok(line)) = reader.lines().next() {
            if line.contains("register") {
                println!("{}", line); // Display the register IP message to user
            }
        }
        // We won't continuously read further output to avoid log flooding
    }
    // 5. Store child handle (or at least PID) for monitoring and later use
    WORKER_CHILD_HANDLE = Some(child);
    println!("Worker started successfully.");
    Ok(())
}


fn stop_worker() -> Result<()> {
    if !is_worker_running() {
        println!("Worker is not running.");
        return Ok(());
    }
    // 1. Send shutdown request via IPC or signal
    if let Some(pid) = stored_worker_pid() {
        // Preferred: send IPC command
        if let Err(e) = send_ipc_command("shutdown") {
            // If IPC fails (no socket?), fallback to SIGTERM
            #[cfg(unix)]
            unsafe { libc::kill(pid, libc::SIGTERM); }
            #[cfg(windows)]
            { /* call TerminateProcess or skip for graceful windows handling */ }
        }
    } else {
        // If no PID (shouldn't happen if running), just attempt IPC
        let _ = send_ipc_command("shutdown");
    }
    // 2. Wait for the Worker to exit gracefully
    for i in 0..10 {
        if !is_worker_running() {
            println!("Worker stopped.");
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    if is_worker_running() {
        println!("Worker did not shut down in time, forcing exit.");
        // Force kill as last resort
        if let Some(pid) = stored_worker_pid() {
            #[cfg(unix)]
            unsafe { libc::kill(pid, libc::SIGKILL); }
            #[cfg(windows)]
            { /* TerminateProcess as last resort */ }
        }
    }
    // 3. Cleanup: remove any state, reset status
    WORKER_CHILD_HANDLE = None;
    Ok(())
}

pub fn view_worker_status() -> anyhow::Result<(), Error>{
    let seassion: Seassion = state.get_session();
    let cfg = config::load_core_config()?;
    let worker_status: WorkerStatusEnum = commands::get_worker_status(seassion, cfg.worker_id); 
    print!("Worker status: [{}]", worker_status.to_string());
}

pub fn view_worker_status() -> anyhow::Result<(), Error>{
    let seassion: Seassion = state.get_session();
    let cfg = config::load_core_config()?;
    let worker_status: WorkerStatusEnum = commands::get_worker_status(seassion, cfg.worker_id); 
    print!("Worker status: [{}]", worker_status.to_string());
}



async fn user_settings_tui() {
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

            let result = crate::commands::update_user(
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
                }
                Err(err) => println!("❌ {}", err),
            }
        }
        _ => {}
    }
}
