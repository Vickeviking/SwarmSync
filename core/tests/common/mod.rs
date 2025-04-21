use chrono::Utc;
use diesel_async::AsyncPgConnection;
use reqwest::{header, Client, ClientBuilder, StatusCode};
use serde_json::json;
use swarmsync_core::commands;
use swarmsync_core::database::models::job::{Job, JobAssignment};
use swarmsync_core::database::models::user::{User, UserResponse};
use swarmsync_core::database::models::worker::Worker;
use swarmsync_core::database::repositories::user::UserRepository;
use swarmsync_core::database::repositories::JobRepository;
use uuid::Uuid;

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
pub async fn create_user_via_api(client: &Client, username: &str) -> UserResponse {
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
        .expect("Failed to send create-user request");

    assert!(
        res.status().is_success(),
        "User creation failed (status={}): {:?}",
        res.status(),
        res.text().await.unwrap_or_default()
    );

    res.json::<UserResponse>()
        .await
        .expect("Failed to deserialize User from create response")
}

/// Fetch full user model directly from the database
pub async fn get_full_user_model(user_id: i32) -> User {
    let mut conn: AsyncPgConnection = commands::load_db_connection().await;
    UserRepository::find_by_id(&mut conn, user_id)
        .await
        .expect("Failed to fetch user by ID")
}

/// Attempt login via HTTP API, returning the response
pub async fn login_user(client: &Client, username: &str, password: &str) -> reqwest::Response {
    client
        .post(&format!("{}/login", APP_HOST))
        .json(&json!({ "username": username, "password": password }))
        .send()
        .await
        .expect("Login request failed")
}

/// Returns a logged-in client and the corresponding user
pub async fn build_client_with_logged_in_admin() -> (Client, UserResponse) {
    let client = http_client();

    let username = generate_unique_username();
    let user = create_user_via_api(&client, &username).await;

    let login_resp = login_user(&client, &username, TEST_PASSWORD).await;
    assert_eq!(login_resp.status(), StatusCode::OK);

    let token = login_resp.json::<serde_json::Value>().await.unwrap()["token"]
        .as_str()
        .unwrap()
        .to_string();

    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
    );

    let authorized_client = ClientBuilder::new()
        .default_headers(headers)
        .build()
        .unwrap();

    (authorized_client, user)
}

/// Delete user via API by ID
pub async fn delete_user_via_api(client: &Client, id: i32) {
    let res = client
        .delete(&format!("{}/users/{}", APP_HOST, id))
        .send()
        .await
        .expect("Delete-user request failed");

    assert!(
        res.status().is_success() || res.status() == StatusCode::NO_CONTENT,
        "Delete-user failed: status={} body={}",
        res.status(),
        res.text().await.unwrap_or_default()
    );
}

/// Delete multiple users
pub async fn delete_users_via_api(client: &Client, user_ids: &[i32]) {
    for &id in user_ids {
        delete_user_via_api(client, id).await;
    }
}

// ========== Job Utilities ==========

pub fn generate_unique_job_name() -> String {
    format!("test_job_{}", Uuid::new_v4())
}

/// Create `n` jobs and return client, user, jobs, job_ids
pub async fn build_client_and_user_with_n_jobs(
    n: usize,
) -> (Client, UserResponse, Vec<Job>, Vec<i32>) {
    let (client, user) = build_client_with_logged_in_admin().await;
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
            .expect("Job creation request failed");

        assert_eq!(resp.status(), StatusCode::CREATED);

        let job: Job = resp.json().await.expect("Failed to parse created job");
        job_ids.push(job.id);
        jobs.push(job);
    }

    (client, user, jobs, job_ids)
}

/// Delete a job by ID
pub async fn delete_job_via_api(client: &Client, job_id: i32) {
    let resp = client
        .delete(&format!("{}/jobs/{}", APP_HOST, job_id))
        .send()
        .await
        .expect("Failed to delete job");

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

/// Delete multiple jobs
pub async fn delete_jobs_via_api(client: &Client, job_ids: &[i32]) {
    for &id in job_ids {
        delete_job_via_api(client, id).await;
    }
}

pub async fn mark_job_running(client: &Client, job_id: i32) {
    //get job
    let mut conn: AsyncPgConnection = commands::load_db_connection().await;
    let job: Job = JobRepository::find_by_id(&mut conn, job_id)
        .await
        .expect("Could not find job through id in db");

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

    let response = client
        .patch(format!("{}/jobs/{job_id}", APP_HOST))
        .json(&updated_payload)
        .send()
        .await
        .expect("Failed to mark job complete");
}

pub async fn mark_job_failed(client: &Client, job_id: i32) {
    //get job
    let mut conn: AsyncPgConnection = commands::load_db_connection().await;
    let job: Job = JobRepository::find_by_id(&mut conn, job_id)
        .await
        .expect("Could not find job through id in db");

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

    let response = client
        .patch(format!("{}/jobs/{job_id}", APP_HOST))
        .json(&updated_payload)
        .send()
        .await
        .expect("Failed to mark job complete");
}

// ======== Worker Utilities ==========
pub async fn create_worker_via_api(client: &Client, user_id: i32) -> Worker {
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
        .expect("Failed to send create-worker request");

    assert!(
        res.status().is_success(),
        "Worker creation failed (status={}): {:?}",
        res.status(),
        res.text().await.unwrap_or_default()
    );

    res.json::<Worker>()
        .await
        .expect("Failed to deserialize Worker from create response")
}

pub async fn assign_job_to_worker(client: &Client, job_id: i32, worker_id: i32) -> JobAssignment {
    let res = client
        .post(&format!("{}/assignments", APP_HOST))
        .json(&json!({
            "job_id": job_id,
            "worker_id": worker_id
        }))
        .send()
        .await
        .expect("Failed to send create-assignment request");

    assert!(
        res.status().is_success(),
        "Job assignment failed (status={}): {:?}",
        res.status(),
        res.text().await.unwrap_or_default()
    );

    res.json::<JobAssignment>()
        .await
        .expect("Failed to deserialize JobAssignment from response")
}

pub async fn delete_worker_via_api(client: &Client, worker_id: i32) {
    let res = client
        .delete(&format!("{}/workers/{}", APP_HOST, worker_id))
        .send()
        .await
        .expect("Failed to send DELETE /workers/:id request");

    assert!(
        res.status().is_success(),
        "Failed to delete worker (status={}): {:?}",
        res.status(),
        res.text().await.unwrap_or_default()
    );
}
