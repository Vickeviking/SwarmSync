use anyhow::{anyhow, ensure, Context};
use chrono::{NaiveDateTime, Utc};
use diesel_async::AsyncPgConnection;
use reqwest::{header, Client, ClientBuilder, StatusCode};
use serde_json::json;
use uuid::Uuid;

use common::commands;
use common::database::models::job::{Job, JobAssignment, JobMetric, JobResult};
use common::database::models::user::UserResponse;
use common::database::models::worker::{Worker, WorkerStatus};
use common::database::repositories::{JobAssignmentRepository, JobRepository};

// ===== UTILITIES =====
pub fn get_ndt_now() -> NaiveDateTime {
    Utc::now().naive_utc()
}

// ========== Constants ==========
pub const APP_HOST: &str = "http://localhost:8000";
pub const TEST_PASSWORD: &str = "ADMINPASSWORD123";

// ========== HTTP Client Setup ==========
pub fn http_client() -> Client {
    Client::new()
}

// ========== User Utilities ==========

pub fn generate_unique_username() -> String {
    format!("test_admin_{}", Uuid::new_v4())
}

/// Create a user via the HTTP API
pub async fn create_user_via_api(
    client: &Client,
    username: &str,
) -> anyhow::Result<UserResponse, anyhow::Error> {
    let email = format!("{}@example.com", username);
    let res = client
        .post(&format!("{}/users", APP_HOST))
        .json(&json!({
            "username": username,
            "email": email,
            "password": TEST_PASSWORD
        }))
        .send()
        .await
        .context("Failed to send create-user request")?;

    assert!(
        res.status().is_success(),
        "User creation failed (status={}): {:?}",
        res.status(),
        res.text().await.unwrap_or_default()
    );

    let user_resp = res
        .json::<UserResponse>()
        .await
        .context("Failed to deserialize User from create response")?;

    Ok(user_resp)
}

/// Attempt login via HTTP API, returning the response
pub async fn login_user(
    client: &Client,
    username: &str,
    password: &str,
) -> anyhow::Result<reqwest::Response, anyhow::Error> {
    let resp = client
        .post(&format!("{}/login", APP_HOST))
        .json(&json!({ "username": username, "password": password }))
        .send()
        .await
        .context("Login request failed")?;

    Ok(resp)
}

/// Returns a logged-in client and the corresponding user
pub async fn build_client_with_logged_in_admin(
) -> anyhow::Result<(Client, UserResponse), anyhow::Error> {
    let client = http_client();

    let username = generate_unique_username();
    let user = create_user_via_api(&client, &username)
        .await
        .context("Failed to create user with api")?;

    let login_resp = login_user(&client, &username, TEST_PASSWORD).await?;

    ensure!(login_resp.status() == StatusCode::OK,);

    let token = login_resp.json::<serde_json::Value>().await.unwrap()["token"]
        .as_str()
        .ok_or_else(|| anyhow!("could not extract login token"))?
        .to_string();

    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&format!("Bearer {}", token)).context("invalid tokens")?,
    );

    let authorized_client = ClientBuilder::new().default_headers(headers).build()?;

    Ok((authorized_client, user))
}

/// Delete user via API by ID
pub async fn delete_user_via_api(client: &Client, id: i32) -> anyhow::Result<(), anyhow::Error> {
    let res = client
        .delete(&format!("{}/users/{}", APP_HOST, id))
        .send()
        .await
        .context("Delete-user request failed")?;

    assert!(
        res.status().is_success() || res.status() == StatusCode::NO_CONTENT,
        "Delete-user failed: status={} body={}",
        res.status(),
        res.text().await.unwrap_or_default()
    );

    Ok(())
}

/// Delete multiple users
pub async fn delete_users_via_api(
    client: &Client,
    user_ids: &[i32],
) -> anyhow::Result<(), anyhow::Error> {
    for &id in user_ids {
        delete_user_via_api(client, id)
            .await
            .context("Failed to delete user via api")?;
    }

    Ok(())
}

// ========== Job Utilities ==========

pub fn generate_unique_job_name() -> String {
    format!("test_job_{}", Uuid::new_v4())
}

/// Create `n` jobs and return client, user, jobs, job_ids
pub async fn build_client_and_user_with_n_jobs(
    n: usize,
) -> anyhow::Result<(Client, UserResponse, Vec<Job>, Vec<i32>), anyhow::Error> {
    let (client, user) = build_client_with_logged_in_admin().await?;
    let mut jobs = Vec::with_capacity(n);
    let mut job_ids = Vec::with_capacity(n);

    for _ in 0..n {
        let payload = json!({
            "user_id": user.id,
            "job_name": generate_unique_job_name(),
            "image_url": "docker.io/library/alpine:latest",
            "image_format": "DockerRegistry",
            "docker_flags": null,
            "output_type": "Stdout",
            "output_paths": null,
            "schedule_type": "Once",
            "cron_expression": null,
            "notes": null,
            "state": "Queued"
        });

        let resp = client
            .post(format!("{}/jobs", APP_HOST))
            .json(&payload)
            .send()
            .await
            .context("Job creation request failed")?;

        assert_eq!(resp.status(), StatusCode::CREATED);

        let job: Job = resp.json().await.expect("Failed to parse created job");
        job_ids.push(job.id);
        jobs.push(job);
    }

    Ok((client, user, jobs, job_ids))
}

/// Delete a job by ID
pub async fn delete_job_via_api(client: &Client, job_id: i32) -> anyhow::Result<(), anyhow::Error> {
    let resp = client
        .delete(&format!("{}/jobs/{}", APP_HOST, job_id))
        .send()
        .await
        .context("Failed to delete job")?;

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    Ok(())
}

/// Delete multiple jobs
pub async fn delete_jobs_via_api(
    client: &Client,
    job_ids: &[i32],
) -> anyhow::Result<(), anyhow::Error> {
    for &id in job_ids {
        delete_job_via_api(client, id).await?;
    }
    Ok(())
}

/// Mark a job as running, use pg connection and bypass rocket routes
/// # Arguments
/// * `job_id` - The ID of the job to mark as running
/// * `client` - The client, must be logged in
/// # Returns
/// * `anyhow::Result<(), anyhow::Error>`
/// # Example
/// ```
/// let client = build_client_with_logged_in_admin().await;
/// mark_job_running(&client, 1).await;
/// ```
pub async fn mark_job_running(client: &Client, job_id: i32) -> anyhow::Result<(), anyhow::Error> {
    //get job
    let mut conn: AsyncPgConnection = commands::load_db_connection()
        .await
        .context("Failed to load db connection")?;

    let job: Job = JobRepository::find_by_id(&mut conn, job_id)
        .await
        .context("Could not find job through id in db")?;

    let updated_payload = serde_json::json!({
        "id": job_id,
        "user_id": job.user_id,
        "job_name": job.job_name,
        "image_url": job.image_url,
        "image_format": job.image_format,
        "docker_flags": job.docker_flags,
        "output_type": job.output_type,
        "output_paths": job.output_paths,
        "schedule_type": job.schedule_type,
        "cron_expression": job.cron_expression,
        "notes": job.notes,
        "state": "Running",
        "created_at": job.created_at,
        "updated_at": Utc::now().naive_utc(),
    });

    let _ = client
        .patch(format!("{}/jobs/{job_id}", APP_HOST))
        .json(&updated_payload)
        .send()
        .await
        .context("Failed to mark job complete")?;

    Ok(())
}

/// Mark a job as failed through direct postgres API call bypassing rocket routes
/// # Arguments
/// * `job_id` - The ID of the job
/// * `client` - The client, must be logged in
/// # Returns
/// * `anyhow::Result<(), anyhow::Error>`
/// # Example
/// ```
/// let client = build_client_with_logged_in_admin().await;
/// mark_job_failed(&client, 1).await;
/// ```
pub async fn mark_job_failed(client: &Client, job_id: i32) -> anyhow::Result<(), anyhow::Error> {
    //get job
    let mut conn: AsyncPgConnection = commands::load_db_connection()
        .await
        .context("Failed to load db connection")?;
    let job: Job = JobRepository::find_by_id(&mut conn, job_id)
        .await
        .context("Could not find job through id in db")?;

    let updated_payload = serde_json::json!({
        "id": job_id,
        "user_id": job.user_id,
        "job_name": job.job_name,
        "image_url": job.image_url,
        "image_format": job.image_format,
        "docker_flags": job.docker_flags,
        "output_type": job.output_type,
        "output_paths": job.output_paths,
        "schedule_type": job.schedule_type,
        "cron_expression": job.cron_expression,
        "notes": job.notes,
        "state": "Failed",
        "created_at": job.created_at,
        "updated_at": Utc::now().naive_utc(),
    });

    let _ = client
        .patch(format!("{}/jobs/{job_id}", APP_HOST))
        .json(&updated_payload)
        .send()
        .await
        .context("Failed to mark job complete")?;

    Ok(())
}

/// Mark an assignment as started via the API, bypassing rocket routes
/// # Arguments
/// * `client` - The client, must be logged in
/// * `assignment_id` - The ID of the assignment
/// * `started_at` - The time the assignment was started
/// # Returns
/// * `anyhow::Result<(), anyhow::Error>`
/// # Example
/// ```
/// let client = build_client_with_logged_in_admin().await;
/// let dt = Utc::now().naive_utc();
/// mark_assignment_started_via_api(&client, 1, dt).await;
/// ```
pub async fn mark_assignment_started_via_api(
    client: &Client,
    assignment_id: i32,
    started_at: NaiveDateTime,
) -> anyhow::Result<(), anyhow::Error> {
    // Fetch the assignment details
    let mut conn: AsyncPgConnection = commands::load_db_connection()
        .await
        .context("Failed to load db connection")?;
    let assignment: JobAssignment = JobAssignmentRepository::find_by_id(&mut conn, assignment_id)
        .await
        .context("Could not find assignment by id")?;

    // Construct the payload to update the assignment
    let updated_payload = json!({
        "id": assignment.id,
        "job_id": assignment.job_id,
        "worker_id": assignment.worker_id,
        "assigned_at": assignment.assigned_at,
        "started_at": started_at.to_string(),
        "finished_at": assignment.finished_at,
    });

    // Send the PATCH request
    let resp = client
        .patch(&format!(
            "{}/assignments/{}/started",
            APP_HOST, assignment_id
        ))
        .json(&updated_payload)
        .send()
        .await
        .context("Failed to mark assignment started")?;

    // Assert that the request was successful
    assert_eq!(resp.status(), StatusCode::OK);

    Ok(())
}

/// Mark a job assignment as finished via API
pub async fn mark_assignment_finished_via_api(
    client: &Client,
    assignment_id: i32,
    finished_at: NaiveDateTime,
) -> anyhow::Result<(), anyhow::Error> {
    // Fetch the assignment details
    let mut conn: AsyncPgConnection = commands::load_db_connection()
        .await
        .context("Failed to load db connection")?;
    let assignment: JobAssignment = JobAssignmentRepository::find_by_id(&mut conn, assignment_id)
        .await
        .context("Could not find assignment by id")?;

    // Construct the payload to update the assignment
    let updated_payload = json!({
        "id": assignment.id,
        "job_id": assignment.job_id,
        "worker_id": assignment.worker_id,
        "assigned_at": assignment.assigned_at,
        "started_at": assignment.started_at,
        "finished_at": finished_at.to_string(), // Adding the finished_at value
    });

    // Send the PATCH request
    let resp = client
        .patch(&format!(
            "{}/assignments/{}/finished",
            APP_HOST, assignment_id
        ))
        .json(&updated_payload)
        .send()
        .await
        .context("Failed to mark assignment finished")?;

    // Assert that the request was successful
    assert_eq!(resp.status(), StatusCode::OK);

    Ok(())
}

// Metrics
pub async fn create_metric_via_api(
    client: &Client,
    job_id: i32,
    worker_id: i32,
    duration_sec: i32,
    cpu_usage_pct: f32,
    mem_usage_mb: f32,
    exit_code: i32,
) -> anyhow::Result<JobMetric, anyhow::Error> {
    let payload = json!({
        "job_id": job_id,
        "worker_id": worker_id,
        "duration_sec": duration_sec,
        "cpu_usage_pct": cpu_usage_pct,
        "mem_usage_mb": mem_usage_mb,
        "exit_code": exit_code
    });

    let res = client
        .post(&format!("{}/metrics", APP_HOST))
        .json(&payload)
        .send()
        .await
        .context("Failed to send create-metric request")?;

    assert!(
        res.status().is_success(),
        "Metric creation failed (status={}): {:?}",
        res.status(),
        res.text().await.unwrap_or_default()
    );

    let job_metric = res
        .json::<JobMetric>()
        .await
        .context("Failed to deserialize JobMetric from create response")?;

    Ok(job_metric)
}

// JOb result
pub async fn assign_result_to_job(
    client: &Client,
    job_id: i32,
) -> anyhow::Result<JobResult, anyhow::Error> {
    let payload = json!({
        "job_id": job_id,
        "stdout": Some("Execution finished successfully."),
        "files": vec![
            "result.log".to_string(),
            "output.json".to_string()
        ]
    });

    let res = client
        .post(&format!("{}/results", APP_HOST))
        .json(&payload)
        .send()
        .await
        .context("Failed to send POST /results")?;

    assert!(
        res.status().is_success(),
        "JobResult creation failed (status={}): {:?}",
        res.status(),
        res.text().await.unwrap_or_default()
    );

    let job_result = res
        .json::<JobResult>()
        .await
        .context("Failed to deserialize JobResult")?;

    Ok(job_result)
}

// ======== Worker Utilities ==========
pub async fn create_worker_via_api(
    client: &Client,
    user_id: i32,
) -> anyhow::Result<Worker, anyhow::Error> {
    let label = format!("worker-{}", Uuid::new_v4());
    let res = client
        .post(&format!("{}/workers", APP_HOST))
        .json(&json!({
            "user_id": user_id,
            "label": label,
            "ip_address": "127.0.0.1",
            "hostname": "test-host",
            "ssh_user": "test-user",
            "ssh_key": "ssh-rsa AAA...",
            "docker_version": "24.0.2",
            "arch": "x86_64",
            "os": "Linux",
            "tags": ["test", "integration"]
        }))
        .send()
        .await
        .context("Failed to send create-worker request")?;

    assert!(
        res.status().is_success(),
        "Worker creation failed (status={}): {:?}",
        res.status(),
        res.text().await.unwrap_or_default()
    );

    let worker = res
        .json::<Worker>()
        .await
        .context("Failed to deserialize Worker from create response")?;

    Ok(worker)
}

pub async fn assign_job_to_worker(
    client: &Client,
    job_id: i32,
    worker_id: i32,
) -> anyhow::Result<JobAssignment, anyhow::Error> {
    let res = client
        .post(&format!("{}/assignments", APP_HOST))
        .json(&json!({
            "job_id": job_id,
            "worker_id": worker_id
        }))
        .send()
        .await
        .context("Failed to send create-assignment request")?;

    assert!(
        res.status().is_success(),
        "Job assignment failed (status={}): {:?}",
        res.status(),
        res.text().await.unwrap_or_default()
    );

    let job_assignment = res
        .json::<JobAssignment>()
        .await
        .context("Failed to deserialize JobAssignment from response")?;

    Ok(job_assignment)
}

pub async fn delete_worker_via_api(
    client: &Client,
    worker_id: i32,
) -> anyhow::Result<(), anyhow::Error> {
    let res = client
        .delete(&format!("{}/workers/{}", APP_HOST, worker_id))
        .send()
        .await
        .context("Failed to send DELETE /workers/:id request")?;

    assert!(
        res.status().is_success(),
        "Failed to delete worker (status={}): {:?}",
        res.status(),
        res.text().await.unwrap_or_default()
    );

    Ok(())
}

pub async fn create_worker_status_via_api(
    client: &Client,
    worker_id: i32,
    job_id: Option<i32>,
) -> anyhow::Result<WorkerStatus, anyhow::Error> {
    let status_data = json!({
        "worker_id": worker_id,
        "status": "Idle",
        "last_heartbeat": null,
        "active_job_id": job_id,
        "uptime_sec": 3600,
        "load_avg": [Some(0.5), Some(0.3), Some(0.1)],
        "last_error": "No errors"
    });

    let res = client
        .post(&format!("{}/worker-status", APP_HOST))
        .json(&status_data)
        .send()
        .await
        .context("Failed to send request to create worker status")?;

    ensure!(res.status() == StatusCode::CREATED,);

    let worker_status = res
        .json::<WorkerStatus>()
        .await
        .context("Failed to parse created WorkerStatus")?;

    Ok(worker_status)
}
