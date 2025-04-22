// src/cli/views/jobs.rs
use crate::cli::utils::{select_assignment, select_job, select_user, select_worker};
use crate::commands;
use crate::shared::enums::{
    image_format::ImageFormatEnum, output::OutputTypeEnum, schedule::ScheduleTypeEnum,
};
use dialoguer::{theme::ColorfulTheme, Input, Select};
pub async fn user_perspective_menu() -> anyhow::Result<()> {
    loop {
        let options = vec![
            "Back",
            "Job CRUD",
            "Worker CRUD",
            "Job-work Assignments",
            "Visualize Job Assignments",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("User Perspective")
            .default(0)
            .items(&options)
            .interact()?;

        match selection {
            0 => break,
            1 => job_crud().await?,
            2 => worker_crud().await?,
            3 => assign_job().await?,
            4 => visualize_jobs().await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

async fn job_crud() -> anyhow::Result<()> {
    let user_id: i32 = select_user().await.unwrap();

    let options = vec!["Back", "List Jobs", "Create Job", "Delete Job"];
    loop {
        let choice = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Job CRUD Menu")
            .default(0)
            .items(&options)
            .interact()?;

        match choice {
            0 => break,
            1 => commands::list_jobs_by_user(user_id).await,
            2 => {
                let job_name: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Job name")
                    .interact_text()?;
                let docker_image: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Docker image")
                    .interact_text()?;

                let image_format = match Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select Image Format")
                    .items(&["Tarball", "DockerRegistry"])
                    .interact()?
                {
                    0 => ImageFormatEnum::Tarball,
                    1 => ImageFormatEnum::DockerRegistry,
                    _ => unreachable!(),
                };

                let output_type = match Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select Output Type")
                    .items(&["Stdout", "Files"])
                    .interact()?
                {
                    0 => OutputTypeEnum::Stdout,
                    1 => OutputTypeEnum::Files,
                    _ => unreachable!(),
                };

                let output_paths = if output_type == OutputTypeEnum::Files {
                    let path: String = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("Enter comma-separated file paths")
                        .interact_text()?;
                    Some(
                        path.split(',')
                            .map(|s| Some(s.trim().to_string()))
                            .collect(),
                    )
                } else {
                    None
                };

                let schedule_type = match Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select Schedule Type")
                    .items(&["Once", "Cron"])
                    .interact()?
                {
                    0 => ScheduleTypeEnum::Once,
                    1 => ScheduleTypeEnum::Cron,
                    _ => unreachable!(),
                };

                let cron_expression = if schedule_type == ScheduleTypeEnum::Cron {
                    let expr: String = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("Enter cron expression")
                        .interact_text()?;
                    Some(expr)
                } else {
                    None
                };

                commands::create_full_job(
                    user_id,
                    job_name,
                    docker_image,
                    image_format,
                    output_type,
                    output_paths,
                    schedule_type,
                    cron_expression,
                )
                .await;
            }
            3 => {
                let job_id: i32 = select_job(user_id).await.unwrap();
                commands::remove_job(job_id).await;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}

async fn worker_crud() -> anyhow::Result<()> {
    let user_id: i32 = select_user().await.unwrap();

    let options = vec![
        "Back",
        "List Workers",
        "Create Worker",
        "Update Worker Label",
        "Delete Worker",
    ];
    loop {
        let choice = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Worker CRUD Menu")
            .default(0)
            .items(&options)
            .interact()?;

        match choice {
            0 => break,
            1 => commands::list_workers_by_user(user_id, 10, 0).await,
            2 => {
                let label: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Worker label")
                    .interact_text()?;
                commands::create_worker(user_id, label).await;
            }
            3 => {
                let id: i32 = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Worker ID")
                    .interact_text()?;
                let label: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("New Label")
                    .interact_text()?;
                commands::update_worker(id, label).await;
            }
            4 => {
                let id: i32 = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Worker ID")
                    .interact_text()?;
                commands::delete_worker(id).await;
            }
            _ => unreachable!(),
        }
    }
    Ok(())
}

async fn assign_job() -> anyhow::Result<()> {
    let options = vec!["Back", "Assign Job", "Delete Assignment"];

    loop {
        let choice = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Assignment Menu")
            .default(0)
            .items(&options)
            .interact()?;

        match choice {
            0 => break,
            1 => {
                let user_id: i32 = select_user().await.unwrap();
                let job_id: i32 = select_job(user_id).await.unwrap();
                let worker_id: i32 = select_worker(user_id).await.unwrap();
                commands::assign_job_to_worker(job_id, worker_id).await;
            }
            2 => {
                let assignment_id: i32 = select_assignment().await.unwrap();
                commands::delete_assignment(assignment_id).await;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}

async fn visualize_jobs() -> anyhow::Result<()> {
    println!("[stub] Visualizing job to worker assignments");
    Ok(())
}
