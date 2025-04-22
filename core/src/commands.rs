use crate::api::auth;
use crate::database::models::job::{Job, JobAssignment, NewJob, NewJobAssignment};
use crate::database::models::user::{NewUser, User};
use crate::database::models::worker::{NewWorker, Worker};
use crate::database::repositories::{
    JobAssignmentRepository, JobRepository, UserRepository, WorkerRepository,
};
use crate::shared::enums::image_format::ImageFormatEnum;
use crate::shared::enums::job::JobStateEnum;
use crate::shared::enums::output::OutputTypeEnum;
use crate::shared::enums::schedule::ScheduleTypeEnum;
use crate::shared::enums::workers::OSEnum;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use diesel_async::{AsyncConnection, AsyncPgConnection};

pub async fn load_db_connection() -> AsyncPgConnection {
    let database_url = std::env::var("DATABASE_URL").expect("Cannot load DB url from environment");
    AsyncPgConnection::establish(&database_url)
        .await
        .expect("Cannot connect to Postgres")
}

pub async fn create_user(username: String, email: String, password: String) {
    let mut c = load_db_connection().await;

    let password_hash = auth::hash_password(password).unwrap();
    let new_user = NewUser {
        username,
        email,
        password_hash,
    };

    let created = UserRepository::create(&mut c, new_user).await.unwrap();
    println!("✅ Created user: {} ({})", created.username, created.email);
}

pub async fn get_user_by_id(id: i32) -> Result<User, anyhow::Error> {
    let mut c = load_db_connection().await;
    let user = UserRepository::find_by_id(&mut c, id).await?;
    Ok(user)
}

pub async fn list_users(limit: i64, offset: i64) {
    let mut c = load_db_connection().await;

    let users = UserRepository::list_all(&mut c, limit, offset)
        .await
        .unwrap();
    println!("📄 Listing users (limit: {}, offset: {}):", limit, offset);
    for user in users {
        println!("({})- {} <{}>", user.id, user.username, user.email);
    }
}

pub async fn update_user(id: i32, username: String, email: String, password: String) {
    let mut c = load_db_connection().await;

    //time placeholder
    let d = NaiveDate::from_ymd_opt(2004, 1, 9).unwrap();
    let t = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    let dt = NaiveDateTime::new(d, t);

    let password_hash = auth::hash_password(password).unwrap();
    let user = User {
        id,
        username,
        email,
        password_hash,
        created_at: dt,
    };

    let updated = UserRepository::update(&mut c, id, user).await.unwrap();
    println!(
        "✏️ Updated user {} -> {} ({})",
        id, updated.username, updated.email
    );
}

pub async fn delete_user(id: i32) {
    let mut c = load_db_connection().await;

    let deleted = UserRepository::delete(&mut c, id).await.unwrap();
    if deleted > 0 {
        println!("🗑️ Deleted user with id {}", id);
    } else {
        println!("⚠️ No user found with id {}", id);
    }
}

pub async fn delete_many_users(start: i32, end: i32) {
    let mut c = load_db_connection().await;

    for id in start..=end {
        match UserRepository::delete(&mut c, id).await {
            Ok(0) => println!("⚠️ No user found with id {}", id),
            Ok(1) => println!("🗑️ Deleted user with id {}", id),
            Ok(n) => println!("🗑️ Deleted {} users with id {} (unexpected)", n, id),
            Err(e) => println!("❌ Error deleting user {}: {}", id, e),
        }
    }
}

// ======= JOBS =========

pub async fn create_full_job(
    user_id: i32,
    job_name: String,
    image_url: String,
    image_format: ImageFormatEnum,
    output_type: OutputTypeEnum,
    output_paths: Option<Vec<Option<String>>>,
    schedule_type: ScheduleTypeEnum,
    cron_expression: Option<String>,
) {
    let mut c = load_db_connection().await;

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
        state: JobStateEnum::Queued,
    };

    match JobRepository::create(&mut c, new_job).await {
        Ok(job) => println!("✅ Created job '{}' (id: {})", job.job_name, job.id),
        Err(e) => eprintln!("❌ Failed to create job: {}", e),
    }
}

pub async fn list_jobs_by_user(user_id: i32) {
    let mut c = load_db_connection().await;

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
}

pub async fn remove_job(job_id: i32) {
    let mut c = load_db_connection().await;
    match JobRepository::delete(&mut c, job_id).await {
        Ok(n) if n > 0 => println!("🗑️ Deleted job with ID {}", job_id),
        Ok(_) => println!("⚠️ No job found with ID {}", job_id),
        Err(e) => eprintln!("❌ Error deleting job {}: {}", job_id, e),
    }
}

// ======= WORKERS =========

pub async fn create_worker(user_id: i32, label: String) {
    let mut c = load_db_connection().await;

    let new_worker = NewWorker {
        user_id,
        label,
        ip_address: "127.0.0.1".to_string(),
        hostname: "localhost".to_string(),
        ssh_user: "root".to_string(),
        ssh_key: "~/.ssh/id_rsa".to_string(),
        docker_version: "20.10.7".to_string(),
        arch: "x86_64".to_string(),
        os: OSEnum::Linux,
        tags: None,
    };

    match WorkerRepository::create(&mut c, new_worker).await {
        Ok(worker) => println!("✅ Created worker '{}' (id: {})", worker.label, worker.id),
        Err(e) => eprintln!("❌ Failed to create worker: {}", e),
    }
}

pub async fn update_worker(worker_id: i32, label: String) {
    let mut c = load_db_connection().await;

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
}

pub async fn delete_worker(worker_id: i32) {
    let mut c = load_db_connection().await;

    match WorkerRepository::delete_worker(&mut c, worker_id).await {
        Ok(n) if n > 0 => println!("🗑️ Deleted worker ID {}", worker_id),
        Ok(_) => println!("⚠️ No worker found with ID {}", worker_id),
        Err(e) => eprintln!("❌ Failed to delete worker {}: {}", worker_id, e),
    }
}

pub async fn list_workers_by_user(user_id: i32, limit: i64, offset: i64) {
    let mut c = load_db_connection().await;
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
}

// ======== Job - worker assignment ==========
pub async fn delete_assignment(assignment_id: i32) {
    let mut c = load_db_connection().await;

    match JobAssignmentRepository::delete(&mut c, assignment_id).await {
        Ok(n) if n > 0 => println!("🗑️ Deleted assignment with ID {}", assignment_id),
        Ok(_) => println!("⚠️ No assignment found with ID {}", assignment_id),
        Err(e) => eprintln!("❌ Failed to delete assignment {}: {}", assignment_id, e),
    }
}

pub async fn assign_job_to_worker(job_id: i32, worker_id: i32) {
    let mut c = load_db_connection().await;

    let new_assignment = NewJobAssignment { job_id, worker_id };

    match JobAssignmentRepository::create(&mut c, new_assignment).await {
        Ok(assignment) => println!(
            "✅ Assigned job {} to worker {} (assignment ID: {})",
            assignment.job_id, assignment.worker_id, assignment.id
        ),
        Err(e) => eprintln!("❌ Failed to assign job: {}", e),
    }
}

pub async fn get_jobs_for_user(user_id: i32) -> Result<Vec<Job>, anyhow::Error> {
    let mut c = load_db_connection().await;
    let jobs = JobRepository::list_by_admin(&mut c, user_id, 100, 0).await?;
    Ok(jobs)
}

pub async fn get_workers_for_user(user_id: i32) -> Result<Vec<Worker>, anyhow::Error> {
    let mut c = load_db_connection().await;
    let workers = WorkerRepository::find_by_admin_id(&mut c, user_id).await?;
    Ok(workers)
}

pub async fn get_assignments_for_user(user_id: i32) -> Result<Vec<JobAssignment>, anyhow::Error> {
    let mut c = load_db_connection().await;

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

pub async fn list_assignments_filtered(user_id: i32, job_id: Option<i32>, worker_id: Option<i32>) {
    let mut c = load_db_connection().await;

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
        .filter(|a| job_id.map_or(true, |jid| a.job_id == jid))
        .filter(|a| worker_id.map_or(true, |wid| a.worker_id == wid))
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
}
