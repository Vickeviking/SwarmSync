use crate::cli::graph_tui;
use crate::cli::utils;
use crate::commands;
use crate::database::models::{job::Job, job::JobAssignment, worker::Worker};
use dialoguer::Input;

pub async fn visualize() -> anyhow::Result<()> {
    let user_id: i32 = utils::select_user().await.unwrap();

    let user = commands::get_user_by_id(user_id).await.unwrap_or_else(|_| {
        println!("❌ Failed to fetch user");
        std::process::exit(1);
    });

    let jobs: Vec<Job> = commands::get_jobs_for_user(user_id)
        .await
        .unwrap_or_default();
    let workers: Vec<Worker> = commands::get_workers_for_user(user_id)
        .await
        .unwrap_or_default();
    let assignments: Vec<JobAssignment> = commands::get_assignments_for_user(user_id)
        .await
        .unwrap_or_default();

    graph_tui::launch_graph_tui_with_data(&user.username, &jobs, &workers, &assignments)?;
    Ok(())
}
