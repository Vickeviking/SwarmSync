use dialoguer::{theme::ColorfulTheme, Select};

use crate::{commands, state};

/// Main menu loop presenting user actions and handling navigation.
/// # Returns
/// * `anyhow::Result<()>`
/// # Panics
/// Does not panic.
pub async fn main_menu() -> anyhow::Result<()> {
    loop {
        // Define the menu options
        let options = vec![
            "User Settings",
            "Submit Job",
            "List Jobs",
            "Finished Jobs",
            "Logout",
            "Quit",
        ];
        let choice = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Main Menu")
            .items(&options)
            .default(0)
            .interact()?;

        match choice {
            0 => {
                // ====== USER SETTINGS ======
                let sub_opts = vec!["Change Core / IP", "Update Profile", "Back"];
                let sel = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("User Settings")
                    .items(&sub_opts)
                    .default(0)
                    .interact()?;

                match sel {
                    0 => {
                        // Call connect flow, allow user to change core location
                        let new_base = crate::views::connect::choose_core_location().await?;
                        let new_session = crate::views::auth::auth_flow(&new_base).await?;
                        // Save new session
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
                continue; // back to main loop
            }
            1 => {
                // Submit a new job
                let session = state::get_session();
                // Prompt for job details
                let job_name: String = dialoguer::Input::new()
                    .with_prompt("Job name")
                    .interact_text()?;
                let image_url: String = dialoguer::Input::new()
                    .with_prompt("Docker image (e.g. user/repo:tag or image tarball path)")
                    .interact_text()?;
                // Image format selection
                let img_formats = vec!["DockerRegistry", "Tarball"];
                let img_choice = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Image format")
                    .items(&img_formats)
                    .default(0)
                    .interact()?;
                let image_format = img_formats[img_choice];
                // Output type selection
                let output_types = vec!["Stdout", "Files"];
                let out_choice = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Output type")
                    .items(&output_types)
                    .default(0)
                    .interact()?;
                let output_type = output_types[out_choice];
                // If output type is Files, ask for output paths
                let output_paths = if output_type == "Files" {
                    let paths_input: String = dialoguer::Input::new()
                        .with_prompt("Enter output file paths (comma-separated)")
                        .interact_text()?;
                    // Parse comma-separated paths into a vector, or None if input is empty
                    let paths: Vec<String> = paths_input
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    if paths.is_empty() {
                        None
                    } else {
                        Some(paths)
                    }
                } else {
                    None
                };
                // Schedule type selection
                let sched_types = vec!["Once", "Cron"];
                let sched_choice = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Schedule type")
                    .items(&sched_types)
                    .default(0)
                    .interact()?;
                let schedule_type = sched_types[sched_choice];
                // If Cron schedule, prompt for cron expression
                let cron_expression = if schedule_type == "Cron" {
                    let mut expr: String;
                    loop {
                        expr = dialoguer::Input::new()
                            .with_prompt("Cron expression (e.g. \"0 5 * * *\" for 5 AM daily)")
                            .interact_text()?;
                        if expr.trim().is_empty() {
                            println!(
                                "⚠ Cron expression cannot be empty. Please enter a valid schedule."
                            );
                            continue;
                        }
                        break;
                    }
                    Some(expr.trim().to_string())
                } else {
                    None
                };
                // Call the command to submit the job
                match commands::submit_job(
                    &session,
                    &job_name,
                    &image_url,
                    image_format,
                    output_type,
                    output_paths,
                    schedule_type,
                    cron_expression.as_deref(),
                )
                .await
                {
                    Ok(job) => {
                        println!(
                            "✅ Job \"{}\" submitted successfully (ID: {}).",
                            job.job_name, job.id
                        );
                    }
                    Err(err) => {
                        println!("❌ {}", err); // Print friendly error from submit_job
                    }
                }
            }
            2 => {
                // List all jobs for the user
                let session = state::get_session();
                match commands::list_jobs(&session).await {
                    Ok(jobs) => {
                        if jobs.is_empty() {
                            println!("(No jobs found for user {})", session.user.username);
                        } else {
                            println!("📋 Job List (total {} jobs):", jobs.len());
                            for job in jobs {
                                println!("- [{}] {} (State: {})", job.id, job.job_name, job.state);
                            }
                        }
                    }
                    Err(err) => {
                        println!("❌ Failed to retrieve jobs: {}", err);
                    }
                }
            }
            3 => {
                // View finished jobs and their results
                let session = state::get_session();
                match commands::get_finished_jobs(&session).await {
                    Ok(finished_jobs) => {
                        if finished_jobs.is_empty() {
                            println!("(No finished jobs for user {})", session.user.username);
                        } else {
                            println!("✅ Finished Jobs ({}):", finished_jobs.len());
                            for job in finished_jobs {
                                println!(
                                    "\n🔹 Job {} - {} (State: {}):",
                                    job.id, job.job_name, job.state
                                );
                                match commands::get_results_for_job(&session, job.id).await {
                                    Ok(results) => {
                                        if results.is_empty() {
                                            println!("   (No results available for this job)");
                                        } else {
                                            for (i, result) in results.iter().enumerate() {
                                                // Display each result entry
                                                println!("   Result {}:", i + 1);
                                                if let Some(output) = &result.stdout {
                                                    println!("      Stdout: {}", output);
                                                }
                                                if let Some(files) = &result.files {
                                                    println!("      Files: {}", files.join(", "));
                                                }
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        println!("   ❌ Error fetching results: {}", err);
                                    }
                                }
                            }
                        }
                    }
                    Err(err) => {
                        println!("❌ Failed to fetch finished jobs: {}", err);
                    }
                }
            }
            4 => {
                // Logout: terminate the application (could also implement returning to auth, but here we exit)
                println!(
                    "🔒 Logging out. Goodbye, {}!",
                    state::get_session().user.username
                );
                std::process::exit(0);
            }
            5 => {
                // Quit the application
                println!("👋 Exiting application. Goodbye!");
                std::process::exit(0);
            }
            _ => unreachable!(),
        }
    }
}
