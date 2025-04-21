use clap::{value_parser, Arg, Command};
use swarmsync_core::commands;
use tokio;

#[tokio::main]
async fn main() {
    let matches = Command::new("SwarmSync CLI")
        .about("CLI for basic user and job management")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("create")
                .about("Create a new user")
                .arg(Arg::new("username").required(true))
                .arg(Arg::new("email").required(true))
                .arg(Arg::new("password").required(true)),
        )
        .subcommand(
            Command::new("list")
                .about("List users with pagination")
                .arg(
                    Arg::new("limit")
                        .required(true)
                        .value_parser(value_parser!(i64)),
                )
                .arg(
                    Arg::new("offset")
                        .required(true)
                        .value_parser(value_parser!(i64)),
                ),
        )
        .subcommand(
            Command::new("update")
                .about("Update an existing user by ID")
                .arg(
                    Arg::new("id")
                        .required(true)
                        .value_parser(value_parser!(i32)),
                )
                .arg(Arg::new("username").required(true))
                .arg(Arg::new("email").required(true))
                .arg(Arg::new("password").required(true)),
        )
        .subcommand(
            Command::new("delete").about("Delete a user by ID").arg(
                Arg::new("id")
                    .required(true)
                    .value_parser(value_parser!(i32)),
            ),
        )
        .subcommand(
            Command::new("create_job")
                .about("Create a new job for a user")
                .arg(
                    Arg::new("user_id")
                        .required(true)
                        .value_parser(value_parser!(i32)),
                )
                .arg(Arg::new("job_name").required(true))
                .arg(Arg::new("docker_image").required(true))
                .arg(Arg::new("schedule").required(false)),
        )
        .subcommand(
            Command::new("remove_job").about("Remove a job by ID").arg(
                Arg::new("job_id")
                    .required(true)
                    .value_parser(value_parser!(i32)),
            ),
        )
        .subcommand(
            Command::new("list_jobs")
                .about("List jobs assigned to a user")
                .arg(
                    Arg::new("user_id")
                        .required(true)
                        .value_parser(value_parser!(i32)),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("create", args)) => {
            commands::create_user(
                args.get_one::<String>("username").unwrap().to_string(),
                args.get_one::<String>("email").unwrap().to_string(),
                args.get_one::<String>("password").unwrap().to_string(),
            )
            .await
        }

        Some(("list", args)) => {
            commands::list_users(
                *args.get_one::<i64>("limit").unwrap(),
                *args.get_one::<i64>("offset").unwrap(),
            )
            .await
        }

        Some(("update", args)) => {
            commands::update_user(
                *args.get_one::<i32>("id").unwrap(),
                args.get_one::<String>("username").unwrap().to_string(),
                args.get_one::<String>("email").unwrap().to_string(),
                args.get_one::<String>("password").unwrap().to_string(),
            )
            .await
        }

        Some(("delete", args)) => commands::delete_user(*args.get_one::<i32>("id").unwrap()).await,

        Some(("create_job", args)) => {
            commands::create_job(
                *args.get_one::<i32>("user_id").unwrap(),
                args.get_one::<String>("job_name").unwrap().to_string(),
                args.get_one::<String>("docker_image").unwrap().to_string(),
                args.get_one::<String>("schedule").map(|s| s.to_string()),
            )
            .await
        }

        Some(("remove_job", args)) => {
            commands::remove_job(*args.get_one::<i32>("job_id").unwrap()).await
        }

        Some(("list_jobs", args)) => {
            commands::list_jobs_by_user(*args.get_one::<i32>("user_id").unwrap()).await
        }

        _ => {}
    }
}
