use crate::commands::load_db_connection;
use crate::database::repositories::JobAssignmentRepository;
use crate::database::{
    models::{job::Job, user::User, worker::Worker},
    repositories::{JobRepository, UserRepository, WorkerRepository},
};
use dialoguer::{theme::ColorfulTheme, Select};

pub async fn select_user() -> Option<i32> {
    let mut c = load_db_connection().await;
    let users = UserRepository::list_all(&mut c, 100, 0).await.ok()?;

    let choices: Vec<String> = users
        .iter()
        .map(|u| format!("{} - {}", u.id, u.username))
        .collect();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose a user")
        .items(&choices)
        .default(0)
        .interact()
        .ok()?;

    Some(users[selection].id)
}

pub async fn select_job(user_id: i32) -> Option<i32> {
    let mut c = load_db_connection().await;
    let jobs = JobRepository::list_by_admin(&mut c, user_id, 100, 0)
        .await
        .ok()?;

    let choices: Vec<String> = jobs
        .iter()
        .map(|j| format!("{} - {}", j.id, j.job_name))
        .collect();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose a job")
        .items(&choices)
        .default(0)
        .interact()
        .ok()?;

    Some(jobs[selection].id)
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

pub async fn select_worker(user_id: i32) -> Option<i32> {
    let mut c = load_db_connection().await;
    let workers = WorkerRepository::list_workers_by_admin(&mut c, user_id, 100, 0)
        .await
        .ok()?;

    let choices: Vec<String> = workers
        .iter()
        .map(|w| format!("{} - {}", w.id, w.label))
        .collect();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose a worker")
        .items(&choices)
        .default(0)
        .interact()
        .ok()?;

    Some(workers[selection].id)
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
