use crate::auth;
use crate::database::models::job::{Job, JobAssignment, NewJob, NewJobAssignment};
use crate::database::models::log::{DBLogEntry, NewDBLogEntry};
use crate::database::models::user::{NewUser, User};
use crate::database::models::worker::{NewWorker, NewWorkerStatus, Worker};
use crate::database::repositories::{
    JobAssignmentRepository, JobRepository, LogEntryRepository, UserRepository, WorkerRepository,
    WorkerStatusRepository,
};
use crate::enums::image_format::ImageFormatEnum;
use crate::enums::job::JobStateEnum;
use crate::enums::log::{LogActionEnum, LogLevelEnum};
use crate::enums::output::OutputTypeEnum;
use crate::enums::schedule::ScheduleTypeEnum;
use crate::enums::system::SystemModuleEnum;
use crate::enums::workers::{OSEnum, WorkerStatusEnum};
use anyhow::Context;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use diesel_async::{AsyncConnection, AsyncPgConnection};

#[allow(clippy::expect_used)]
pub async fn load_db_connection() -> anyhow::Result<AsyncPgConnection, anyhow::Error> {
    let database_url =
        std::env::var("DATABASE_URL").context("Cannot load DB url from environment")?;
    let conn = AsyncPgConnection::establish(&database_url)
        .await
        .context("Cannot connect to Postgres")?;
    Ok(conn)
}

pub async fn create_user(
    username: String,
    email: String,
    password: String,
) -> anyhow::Result<(), anyhow::Error> {
    let mut c = load_db_connection().await?;

    let password_hash = auth::hash_password(password).context("Hashing failed")?;
    let new_user = NewUser {
        username,
        email,
        password_hash,
    };

    let created = UserRepository::create(&mut c, new_user)
        .await
        .context("UserRepository create failed")?;
    println!("✅ Created user: {} ({})", created.username, created.email);
    Ok(())
}

pub async fn get_user_by_id(id: i32) -> Result<User, anyhow::Error> {
    let mut c = load_db_connection().await?;
    let user = UserRepository::find_by_id(&mut c, id).await?;
    Ok(user)
}

pub async fn list_users(limit: i64, offset: i64) -> anyhow::Result<(), anyhow::Error> {
    let mut c = load_db_connection().await?;

    let users = UserRepository::list_all(&mut c, limit, offset)
        .await
        .context("list all failed from UserRepository")?;
    println!("📄 Listing users (limit: {}, offset: {}):", limit, offset);
    for user in users {
        println!("({})- {} <{}>", user.id, user.username, user.email);
    }
    Ok(())
}

pub async fn update_user(
    id: i32,
    username: String,
    email: String,
    password: String,
) -> anyhow::Result<(), anyhow::Error> {
    let mut c = load_db_connection().await?;

    //time placeholder
    let d = NaiveDate::from_ymd_opt(2004, 1, 9).context("invalid time placeholder")?;
    let t = NaiveTime::from_hms_opt(0, 0, 0).context("invalid time placeholder")?;
    let dt = NaiveDateTime::new(d, t);

    let password_hash =
        auth::hash_password(password).context("password hashing failed with argon2")?;
    let user = User {
        id,
        username,
        email,
        password_hash,
        created_at: dt,
    };

    let updated = UserRepository::update(&mut c, id, user)
        .await
        .context("User update failed in UserRepository")?;
    println!(
        "✏️ Updated user {} -> {} ({})",
        id, updated.username, updated.email
    );
    Ok(())
}

pub async fn delete_user(id: i32) -> anyhow::Result<(), anyhow::Error> {
    let mut c = load_db_connection().await?;

    let deleted = UserRepository::delete(&mut c, id)
        .await
        .context("UserRepository delete failed")?;
    if deleted > 0 {
        println!("🗑️ Deleted user with id {}", id);
    } else {
        println!("⚠️ No user found with id {}", id);
    }
    Ok(())
}

pub async fn delete_many_users(start: i32, end: i32) -> anyhow::Result<(), anyhow::Error> {
    let mut c = load_db_connection().await?;

    for id in start..=end {
        match UserRepository::delete(&mut c, id).await {
            Ok(0) => println!("⚠️ No user found with id {}", id),
            Ok(1) => println!("🗑️ Deleted user with id {}", id),
            Ok(n) => println!("🗑️ Deleted {} users with id {} (unexpected)", n, id),
            Err(e) => println!("❌ Error deleting user {}: {}", id, e),
        }
    }

    Ok(())
}

// ======= JOBS =========
#[allow(clippy::too_many_arguments)]
pub async fn create_full_job(
    user_id: i32,
    job_name: String,
    image_url: String,
    image_format: ImageFormatEnum,
    output_type: OutputTypeEnum,
    output_paths: Option<Vec<Option<String>>>,
    schedule_type: ScheduleTypeEnum,
    cron_expression: Option<String>,
) -> anyhow::Result<(), anyhow::Error> {
    let mut c = load_db_connection().await?;

    let new_job = NewJob {
        user_id,
        job_name,
        image_url,
        image_format,
        docker_flags: None,
        output_type,
        output_paths,
        schedule_type,
        cron_expression,
        notes: None,
        state: JobStateEnum::Submitted,
    };

    match JobRepository::create(&mut c, new_job).await {
        Ok(job) => println!("✅ Created job '{}' (id: {})", job.job_name, job.id),
        Err(e) => eprintln!("❌ Failed to create job: {}", e),
    }
    Ok(())
}

pub async fn list_jobs_by_user(user_id: i32) -> anyhow::Result<(), anyhow::Error> {
    let mut c = load_db_connection().await?;

    let limit = 10;
    let offset = 0;
    match JobRepository::list_by_admin(&mut c, user_id, limit, offset).await {
        Ok(jobs) => {
            if jobs.is_empty() {
                println!("📭 No jobs found for user {}", user_id);
            } else {
                for job in jobs {
                    println!("({}) {} - {:?}", job.id, job.job_name, job.state);
                }
            }
        }
        Err(e) => eprintln!("❌ Failed to fetch jobs: {}", e),
    }
    Ok(())
}

pub async fn remove_job(job_id: i32) -> anyhow::Result<(), anyhow::Error> {
    let mut c = load_db_connection().await?;
    match JobRepository::delete(&mut c, job_id).await {
        Ok(n) if n > 0 => println!("🗑️ Deleted job with ID {}", job_id),
        Ok(_) => println!("⚠️ No job found with ID {}", job_id),
        Err(e) => eprintln!("❌ Error deleting job {}: {}", job_id, e),
    }
    Ok(())
}

pub async fn get_job_by_id(id: i32) -> anyhow::Result<Job> {
    let mut conn = load_db_connection().await?;
    Ok(JobRepository::find_by_id(&mut conn, id).await?)
}

// ======= WORKERS =========

pub async fn create_worker(user_id: i32, label: String) -> anyhow::Result<()> {
    // ── 1. Open connection ─────────────────────────────────────────────
    let mut conn: AsyncPgConnection = load_db_connection().await.context("DB connect failed")?;

    // ── 2. Run both inserts in a single transaction ────────────────────
    conn.transaction::<_, anyhow::Error, _>(|tx| {
        Box::pin(async move {
            /* 2.1 insert into `workers` */
            let worker = WorkerRepository::create(
                tx,
                NewWorker {
                    user_id,
                    label,
                    ip_address: "127.0.0.1".into(),
                    hostname: "localhost".into(),
                    ssh_user: "root".into(),
                    ssh_key: "~/.ssh/id_rsa".into(),
                    docker_version: "20.10.7".into(),
                    arch: "x86_64".into(),
                    os: OSEnum::Linux,
                    tags: None,
                },
            )
            .await?;

            /* 2.2 insert initial status row */
            WorkerStatusRepository::create(
                tx,
                NewWorkerStatus {
                    worker_id: worker.id,
                    status: WorkerStatusEnum::Offline, // initial state
                    last_heartbeat: None,
                    active_job_id: None,
                    uptime_sec: None,
                    load_avg: None,
                    last_error: None,
                },
            )
            .await?;

            println!(
                "✅ Created worker '{}' (id: {}), status=Offline",
                worker.label, worker.id
            );
            Ok(())
        })
    })
    .await
}

pub async fn update_worker(worker_id: i32, label: String) -> anyhow::Result<(), anyhow::Error> {
    let mut c = load_db_connection().await?;

    match WorkerRepository::find_by_id(&mut c, worker_id).await {
        Ok(mut worker) => {
            worker.label = label;
            match WorkerRepository::update(&mut c, worker_id, worker).await {
                Ok(updated) => println!("✏️ Updated worker {} -> {}", worker_id, updated.label),
                Err(e) => eprintln!("❌ Failed to update worker: {}", e),
            }
        }
        Err(e) => eprintln!("❌ Worker not found: {}", e),
    }
    Ok(())
}

pub async fn delete_worker(worker_id: i32) -> anyhow::Result<(), anyhow::Error> {
    let mut c = load_db_connection().await?;

    match WorkerRepository::delete_worker(&mut c, worker_id).await {
        Ok(n) if n > 0 => println!("🗑️ Deleted worker ID {}", worker_id),
        Ok(_) => println!("⚠️ No worker found with ID {}", worker_id),
        Err(e) => eprintln!("❌ Failed to delete worker {}: {}", worker_id, e),
    }
    Ok(())
}

pub async fn list_workers_by_user(
    user_id: i32,
    limit: i64,
    offset: i64,
) -> anyhow::Result<(), anyhow::Error> {
    let mut c = load_db_connection().await?;
    match WorkerRepository::list_workers_by_admin(&mut c, user_id, limit, offset).await {
        Ok(workers) => {
            if workers.is_empty() {
                println!("📭 No workers found for user {}", user_id);
            } else {
                for w in workers {
                    println!("({}) {} [{}]", w.id, w.label, w.os);
                }
            }
        }
        Err(e) => eprintln!("❌ Failed to list workers: {}", e),
    }
    Ok(())
}

// ======== Job - worker assignment ==========
pub async fn delete_assignment(assignment_id: i32) -> anyhow::Result<(), anyhow::Error> {
    let mut c = load_db_connection().await?;

    match JobAssignmentRepository::delete(&mut c, assignment_id).await {
        Ok(n) if n > 0 => println!("🗑️ Deleted assignment with ID {}", assignment_id),
        Ok(_) => println!("⚠️ No assignment found with ID {}", assignment_id),
        Err(e) => eprintln!("❌ Failed to delete assignment {}: {}", assignment_id, e),
    }
    Ok(())
}

pub async fn assign_job_to_worker(
    job_id: i32,
    worker_id: i32,
) -> anyhow::Result<(), anyhow::Error> {
    let mut c = load_db_connection().await?;

    let new_assignment = NewJobAssignment { job_id, worker_id };

    match JobAssignmentRepository::create(&mut c, new_assignment).await {
        Ok(assignment) => println!(
            "✅ Assigned job {} to worker {} (assignment ID: {})",
            assignment.job_id, assignment.worker_id, assignment.id
        ),
        Err(e) => eprintln!("❌ Failed to assign job: {}", e),
    }
    Ok(())
}

pub async fn get_jobs_for_user(user_id: i32) -> Result<Vec<Job>, anyhow::Error> {
    let mut c = load_db_connection().await?;
    let jobs = JobRepository::list_by_admin(&mut c, user_id, 100, 0).await?;
    Ok(jobs)
}

pub async fn get_workers_for_user(user_id: i32) -> Result<Vec<Worker>, anyhow::Error> {
    let mut c = load_db_connection().await?;
    let workers = WorkerRepository::find_by_admin_id(&mut c, user_id).await?;
    Ok(workers)
}

pub async fn get_assignments_for_user(user_id: i32) -> Result<Vec<JobAssignment>, anyhow::Error> {
    let mut c = load_db_connection().await?;

    // Get jobs by user first
    let jobs = JobRepository::list_by_admin(&mut c, user_id, 100, 0).await?;
    let job_ids: Vec<i32> = jobs.iter().map(|j| j.id).collect();

    // Get all active assignments
    let all_assignments = JobAssignmentRepository::list_active_assignments(&mut c).await?;

    // Filter assignments that belong to this user's jobs
    let filtered: Vec<JobAssignment> = all_assignments
        .into_iter()
        .filter(|a| job_ids.contains(&a.job_id))
        .collect();

    Ok(filtered)
}

pub async fn get_assignment_id_for_job(job_id: i32) -> anyhow::Result<Option<i32>> {
    let mut conn = load_db_connection().await?;
    let assignments = JobAssignmentRepository::find_by_job_id(&mut conn, job_id).await?;
    Ok(assignments.first().map(|a| a.id))
}

pub async fn list_assignments_filtered(
    user_id: i32,
    job_id: Option<i32>,
    worker_id: Option<i32>,
) -> anyhow::Result<(), anyhow::Error> {
    let mut c = load_db_connection().await?;

    let jobs = JobRepository::list_by_admin(&mut c, user_id, 100, 0)
        .await
        .unwrap_or_default();
    let job_ids: Vec<i32> = jobs.iter().map(|j| j.id).collect();

    let all = JobAssignmentRepository::list_active_assignments(&mut c)
        .await
        .unwrap_or_default();

    let filtered = all
        .into_iter()
        .filter(|a| job_ids.contains(&a.job_id))
        .filter(|a| job_id.is_none_or(|jid| a.job_id == jid))
        .filter(|a| worker_id.is_none_or(|wid| a.worker_id == wid))
        .collect::<Vec<_>>();

    if filtered.is_empty() {
        println!("📭 No matching assignments found.");
    } else {
        println!("🔗 Assignments:");
        for a in filtered {
            println!(
                "- Job ID: {}, Worker ID: {}, Assigned: {}",
                a.job_id, a.worker_id, a.assigned_at
            );
        }
    }

    Ok(())
}

// ======== LOGS =========
/// Create a new log entry
pub async fn create_log_entry(
    level: LogLevelEnum,
    module: SystemModuleEnum,
    action: LogActionEnum,
    expires_at: NaiveDateTime,
    client_ip: Option<String>,
    client_username: Option<String>,
    custom_msg: Option<String>,
) -> anyhow::Result<(), anyhow::Error> {
    let mut conn = load_db_connection().await?;
    let new_entry = NewDBLogEntry {
        level,
        module,
        action,
        expires_at,
        client_connected_ip: client_ip,
        client_connected_username: client_username,
        job_submitted_job_id: None,
        job_submitted_from_module: None,
        job_submitted_to_module: None,
        job_completed_job_id: None,
        job_completed_success: None,
        custom_msg,
    };

    match LogEntryRepository::create(&mut conn, new_entry).await {
        Ok(log) => println!("✅ Created log entry with ID {}", log.id),
        Err(e) => eprintln!("❌ Failed to create log entry: {}", e),
    }
    Ok(())
}

/// Fetch a log entry by its ID
pub async fn get_log_by_id(id: i32) -> anyhow::Result<(), anyhow::Error> {
    let mut conn = load_db_connection().await?;
    match LogEntryRepository::find_by_id(&mut conn, id).await {
        Ok(log) => println!("📄 Log {}: {:?}", id, log),
        Err(e) => eprintln!("❌ Error fetching log {}: {}", id, e),
    }
    Ok(())
}

/// Returns the log entry or an error.
pub async fn fetch_log_entry(id: i32) -> Result<DBLogEntry, anyhow::Error> {
    let mut conn = load_db_connection().await?;
    let entry = LogEntryRepository::find_by_id(&mut conn, id).await?;
    Ok(entry)
}

/// Fetch a page of *all* logs
pub async fn fetch_logs(limit: i64, offset: i64) -> Result<Vec<DBLogEntry>, anyhow::Error> {
    let mut conn = load_db_connection().await?;
    Ok(LogEntryRepository::list_all(&mut conn, limit, offset).await?)
}

/// Fetch a page of logs matching exactly `action`
pub async fn fetch_logs_by_action(
    action: LogActionEnum,
    limit: i64,
    offset: i64,
) -> Result<Vec<DBLogEntry>, anyhow::Error> {
    let mut conn = load_db_connection().await?;
    Ok(LogEntryRepository::list_by_action_exact(&mut conn, action, limit, offset).await?)
}

/// Fetch a page of logs matching exactly `level`
pub async fn fetch_logs_by_level(
    level: LogLevelEnum,
    limit: i64,
    offset: i64,
) -> Result<Vec<DBLogEntry>, anyhow::Error> {
    let mut conn = load_db_connection().await?;
    Ok(LogEntryRepository::list_by_level_exact(&mut conn, level, limit, offset).await?)
}

/// Fetch a page of logs in `module`
pub async fn fetch_logs_by_module(
    module: SystemModuleEnum,
    limit: i64,
    offset: i64,
) -> Result<Vec<DBLogEntry>, anyhow::Error> {
    let mut conn = load_db_connection().await?;
    Ok(LogEntryRepository::find_logs_by_module(&mut conn, module, limit, offset).await?)
}

/// Update an existing log entry
pub async fn update_log_entry(id: i32, updated: DBLogEntry) -> anyhow::Result<(), anyhow::Error> {
    let mut conn = load_db_connection().await?;
    match LogEntryRepository::update(&mut conn, id, updated).await {
        Ok(log) => println!("✏️ Updated log {}: {:?}", id, log),
        Err(e) => eprintln!("❌ Failed to update log {}: {}", id, e),
    }
    Ok(())
}

/// Delete a log entry by ID
pub async fn delete_log_entry(id: i32) -> anyhow::Result<(), anyhow::Error> {
    let mut conn = load_db_connection().await?;
    match LogEntryRepository::delete(&mut conn, id).await {
        Ok(n) if n > 0 => println!("🗑️ Deleted log entry {}", id),
        Ok(_) => println!("⚠️ No log found with ID {}", id),
        Err(e) => eprintln!("❌ Error deleting log {}: {}", id, e),
    }
    Ok(())
}
