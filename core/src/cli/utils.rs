use crate::commands;
use crate::commands::load_db_connection;
use crate::database::repositories::JobAssignmentRepository;
use crate::database::{
    models::job::Job,
    repositories::{JobRepository, UserRepository, WorkerRepository},
};
use crate::shared::enums::job::JobStateEnum;
use anyhow::{Context, Result};
use dialoguer::Input;
use dialoguer::{theme::ColorfulTheme, Select};
use std::io::{self, ErrorKind};

/// Used as return type to either represent a 'back' or successfull select
pub enum SelectMenuResult {
    Chosen(i32),
    Back,
}

/// TUI allowing a selection of a user from a list
///
/// # Returns
/// SelectMenuResult holding either back or user_id
pub async fn select_user() -> Result<SelectMenuResult, anyhow::Error> {
    let mut c = load_db_connection().await;
    let users = UserRepository::list_all(&mut c, 100, 0)
        .await
        .context("Error listing users, list_all failed from UserRepository")?;

    let mut choices: Vec<String> = users
        .iter()
        .map(|u| format!("{} - {}", u.id, u.username))
        .collect();
    let back = vec!["back".to_string()];
    choices.splice(0..0, back);

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose a user")
        .items(&choices)
        .default(0)
        .interact()
        .context("Something went wrong inside UI select")?;

    // if back choosen return 'Back'
    if selection == 0 {
        return Ok(SelectMenuResult::Back);
    }

    // Since we have back , -1 for the offset
    Ok(SelectMenuResult::Chosen(users[selection - 1].id))
}

/// TUI allowing a selection of job from {user_id} owned jobs
///
/// # Arguments
/// * user_id - the user whos jobs is to be choosed from  
///
/// # Returns
/// SelectMenuResult holding either back or 'job_id'
pub async fn select_job(user_id: i32) -> Result<SelectMenuResult, anyhow::Error> {
    let mut c = load_db_connection().await;
    let jobs = JobRepository::list_by_admin(&mut c, user_id, 100, 0)
        .await
        .context("Could not list admins, error calling list_by_admin from JobRepository")?;

    let mut choices: Vec<String> = jobs
        .iter()
        .map(|j| format!("{} - {}", j.id, j.job_name))
        .collect();
    let back = vec!["back".to_string()];
    choices.splice(0..0, back);

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose a job")
        .items(&choices)
        .default(0)
        .interact()
        .context("Error selecting user, Interact() failed")?;

    // if back choosen return 'Back'
    if selection == 0 {
        return Ok(SelectMenuResult::Back);
    }

    // Since we have back , -1 for the offset
    Ok(SelectMenuResult::Chosen(jobs[selection - 1].id))
}

/// TUI allowing a selection of worker from {user_id} owned workers
///
/// # Arguments
/// * user_id - the user whos workers is to be choosed from  
///
/// # Returns
/// SelectMenuResult holding either back or 'worker_id'
pub async fn select_worker(user_id: i32) -> Result<SelectMenuResult, anyhow::Error> {
    let mut c = load_db_connection().await;
    let workers = WorkerRepository::list_workers_by_admin(&mut c, user_id, 100, 0)
        .await
        .context("list_workers_by_admin failed")?;

    let mut choices: Vec<String> = workers
        .iter()
        .map(|w| format!("{} - {}", w.id, w.label))
        .collect();
    let back = vec!["back".to_string()];
    choices.splice(0..0, back);

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose a worker")
        .items(&choices)
        .default(0)
        .interact()
        .context("interact() failed, could not select worker from list in TUI")?;

    // if back choosen return 'Back'
    if selection == 0 {
        return Ok(SelectMenuResult::Back);
    }

    // Since we have back , -1 for the offset
    Ok(SelectMenuResult::Chosen(workers[selection - 1].id))
}

pub async fn select_job_with_any(user_id: i32) -> Option<Option<i32>> {
    let mut c = load_db_connection().await;
    let jobs = JobRepository::list_by_admin(&mut c, user_id, 100, 0)
        .await
        .ok()?;

    let mut choices = vec!["Any".to_string()];
    choices.extend(jobs.iter().map(|j| format!("{} - {}", j.id, j.job_name)));

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose a job (or Any)")
        .items(&choices)
        .default(0)
        .interact()
        .ok()?;

    if selection == 0 {
        Some(None)
    } else {
        Some(Some(jobs[selection - 1].id))
    }
}

pub async fn select_worker_with_any(user_id: i32) -> Option<Option<i32>> {
    let mut c = load_db_connection().await;
    let workers = WorkerRepository::list_workers_by_admin(&mut c, user_id, 100, 0)
        .await
        .ok()?;

    let mut choices = vec!["Any".to_string()];
    choices.extend(workers.iter().map(|w| format!("{} - {}", w.id, w.label)));

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose a worker (or Any)")
        .items(&choices)
        .default(0)
        .interact()
        .ok()?;

    if selection == 0 {
        Some(None)
    } else {
        Some(Some(workers[selection - 1].id))
    }
}

pub async fn select_assignment() -> Option<i32> {
    let mut c = load_db_connection().await;
    let assignments = JobAssignmentRepository::list_active_assignments(&mut c)
        .await
        .ok()?;

    if assignments.is_empty() {
        println!("📭 No active assignments found.");
        return None;
    }

    let choices: Vec<String> = assignments
        .iter()
        .map(|a| {
            format!(
                "ID: {} | Job: {} → Worker: {} | Started: {}",
                a.id,
                a.job_id,
                a.worker_id,
                a.started_at
                    .map(|dt| dt.to_string())
                    .unwrap_or_else(|| "not started".into())
            )
        })
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose an assignment")
        .items(&choices)
        .default(0)
        .interact()
        .ok()?;

    Some(assignments[selection].id)
}

pub async fn move_job_state(job_id: i32) -> anyhow::Result<()> {
    let job = commands::get_job_by_id(job_id).await?;
    if job.state == JobStateEnum::Running {
        println!("⚠️ Job is currently RUNNING.");

        let confirm = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("To move this job, you must first remove its assignment. Continue?")
            .items(&["Cancel", "Remove Assignment"])
            .default(0)
            .interact()?;

        if confirm == 0 {
            println!("❌ Move cancelled.");
            return Ok(());
        }

        if let Some(assign_id) = commands::get_assignment_id_for_job(job_id).await? {
            commands::delete_assignment(assign_id).await;
            println!("🗑️ Assignment removed.");
        } else {
            println!("⚠️ No assignment found. Proceeding.");
        }
    }
    let states = vec!["Submitted", "Queued", "Completed", "Failed"];
    let choice = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Move Job To Which State?")
        .default(0)
        .items(&states)
        .interact()?;

    let result = match states[choice] {
        "Submitted" => mark_submitted(job_id).await,
        "Queued" => mark_queued(job_id).await,
        "Completed" => mark_succeeded(job_id).await,
        "Failed" => {
            let msg: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Failure message?")
                .interact_text()?;
            mark_failed(job_id, &msg).await
        }
        _ => unreachable!(),
    };

    match result {
        Ok(job) => {
            println!("✅ Job moved to {:?} successfully", job.state);
        }
        Err(e) => {
            eprintln!("❌ Failed to move job: {}", e);
        }
    }

    Ok(())
}

pub async fn mark_submitted(id: i32) -> anyhow::Result<Job> {
    let mut conn = load_db_connection().await;
    Ok(JobRepository::mark_submitted(&mut conn, id).await?)
}

pub async fn mark_queued(id: i32) -> anyhow::Result<Job> {
    let mut conn = load_db_connection().await;
    Ok(JobRepository::mark_queued(&mut conn, id).await?)
}

pub async fn mark_running(id: i32) -> anyhow::Result<Job> {
    let mut conn = load_db_connection().await;
    Ok(JobRepository::mark_running(&mut conn, id).await?)
}

pub async fn mark_succeeded(id: i32) -> anyhow::Result<Job> {
    let mut conn = load_db_connection().await;
    Ok(JobRepository::mark_succeeded(&mut conn, id).await?)
}

pub async fn mark_failed(id: i32, message: &str) -> anyhow::Result<Job> {
    let mut conn = load_db_connection().await;
    Ok(JobRepository::mark_failed(&mut conn, id, message).await?)
}
