/*
===================== 🚀 Job API Overview =====================

== 🛠️ CRUD ==
• POST   /jobs                     -> Creates a new job (NewJob)           → 201 Created (Job)
• GET    /jobs/:id                 -> Fetch job by ID                      → 200 OK (Job)
• PATCH  /jobs/:id                 -> Update job by ID (Job)               → 200 OK (Job)
• DELETE /jobs/:id                -> Delete job by ID                     → 204 No Content

== 🔍 Lookup & Search ==
• GET    /jobs/search?user_id&query         -> Fuzzy match jobs by name         → 200 OK (Vec<Job>)
• GET    /jobs/name/:user_id?name           -> Exact match job by name          → 200 OK (Vec<Job>)
• GET    /jobs/by_admin?user_id&limit&offset→ Jobs by a specific admin          → 200 OK (Vec<Job>)
• GET    /jobs/state/:state                 -> Jobs by job state enum           → 200 OK (Vec<Job>)
• GET    /jobs/recent?limit                 -> Most recent jobs (default 10)    → 200 OK (Vec<Job>)
• GET    /jobs/failed?limit                 -> Recently failed jobs             → 200 OK (Vec<Job>)

== 🔄 State Transitions ==
• PATCH  /jobs/:id/running        -> Mark job as running                   → 200 OK (Job)
• PATCH  /jobs/:id/succeeded      -> Mark job as succeeded                 → 200 OK (Job)
• PATCH  /jobs/:id/failed         -> Mark job as failed (with message)     → 200 OK (Job)

== ⏱️ Scheduling & Readiness ==
• GET    /jobs/scheduled                   -> All jobs with a schedule          → 200 OK (Vec<Job>)
• GET    /jobs/cron_due?date&time         -> Cron jobs due at a given time     → 200 OK (Vec<Job>)
• GET    /jobs/ready                       -> One-time jobs ready to run        → 200 OK (Vec<Job>)

== 📊 Aggregation & Stats ==
• GET    /jobs/stats/admins                -> Job count grouped by admin ID     → 200 OK (Vec<(i32, i64)>)

== 🤝 Assignment & Worker Routing ==
• GET    /jobs/active/:worker_id           -> Active jobs for worker            → 200 OK (Vec<Job>)
• GET    /jobs/assigned/:worker_id         -> Jobs assigned to worker           → 200 OK (Vec<Job>)
• GET    /jobs/unassigned                  -> Jobs with no worker assignment    → 200 OK (Vec<Job>)

===============================================================
*/

pub mod common_test;

use common_test::enums::job::JobStateEnum;
use rocket::serde::json::json;

#[cfg(test)]
mod job_api_tests {
    use chrono::Utc;
    use common::database::models::job::Job;
    use reqwest::StatusCode;

    use crate::common_test::APP_HOST;

    use super::*;

    #[tokio::test]
    async fn test_create_job() {
        // Build the authorized client with a logged-in admin user
        let (client, user) = common_test::build_client_with_logged_in_admin().await;

        // Build JSON payload directly
        let payload_json = json!({
            "user_id": user.id,
            "job_name": common_test::generate_unique_job_name(),
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

        // Send the request
        let response = client
            .post(format!("{}/jobs", common_test::APP_HOST))
            .json(&payload_json)
            .send()
            .await
            .expect("Failed to send create job request");

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = response.text().await.expect("Failed to read response body");

        let created_job: Job = serde_json::from_str(&body).unwrap_or_else(|e| {
            panic!(
                "Failed to parse Job from response JSON:\nError: {}\nBody:\n{}",
                e, body
            )
        });

        // Extract values from original JSON to assert against
        let expected_job_name = payload_json["job_name"].as_str().unwrap();
        let expected_image_url = payload_json["image_url"].as_str().unwrap();

        assert_eq!(created_job.user_id, user.id);
        assert_eq!(created_job.job_name, expected_job_name);
        assert_eq!(created_job.image_url, expected_image_url);
        assert_eq!(created_job.image_format.to_string(), "DockerRegistry");
        assert_eq!(created_job.state.to_string(), "Queued");

        common_test::delete_jobs_via_api(&client, &vec![created_job.id]).await;
        common_test::delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_get_job_by_id() {
        let (client, user, jobs, job_ids) = common_test::build_client_and_user_with_n_jobs(3).await;
        let job_id = job_ids[1];

        let response = client
            .get(format!("{}/jobs/{job_id}", common_test::APP_HOST))
            .send()
            .await
            .expect("Failed to send GET /jobs/:id");

        assert_eq!(response.status(), 200);

        let job: Job = response.json().await.expect("Invalid JobPayload response");
        assert_eq!(job.id, job_id);
        assert_eq!(job.job_name, jobs[1].job_name);

        common_test::delete_jobs_via_api(&client, &job_ids).await;
        common_test::delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_update_job() {
        let (client, user, jobs, job_ids) = common_test::build_client_and_user_with_n_jobs(1).await;
        let job_id = job_ids[0];

        let new_job_name = common_test::generate_unique_job_name();
        let job_notes = "Hello this job is cpu bound";

        let updated_payload = serde_json::json!({
            "id": job_id,
            "user_id": user.id,
            "job_name": new_job_name,
            "image_url": "docker.io/library/alpine:latest",
            "image_format": "DockerRegistry",
            "docker_flags": null,
            "output_type": "Stdout",
            "output_paths": null,
            "schedule_type": "Once",
            "cron_expression": null,
            "notes": job_notes,
            "state": "Running",
            "created_at": jobs[0].created_at,
            "updated_at": Utc::now().naive_utc(),
        });

        let response = client
            .patch(format!("{}/jobs/{job_id}", common_test::APP_HOST))
            .json(&updated_payload)
            .send()
            .await
            .expect("Failed to send PATCH /jobs/:id");

        assert_eq!(response.status(), 200);

        let updated_job: Job = response.json().await.expect("Invalid updated JobPayload");
        assert_eq!(updated_job.state, JobStateEnum::Running);
        assert_eq!(updated_job.job_name, new_job_name);
        assert_eq!(updated_job.notes.unwrap(), job_notes);

        common_test::delete_jobs_via_api(&client, &job_ids).await;
        common_test::delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_delete_job() {
        let (client, user, _jobs, job_ids) =
            common_test::build_client_and_user_with_n_jobs(1).await;
        let job_id = job_ids[0];

        let delete_response = client
            .delete(format!("{}/jobs/{job_id}", common_test::APP_HOST))
            .send()
            .await
            .expect("Failed to send DELETE /jobs/:id");

        assert_eq!(delete_response.status(), 204);

        let get_response = client
            .get(format!("{}/jobs/{job_id}", common_test::APP_HOST))
            .send()
            .await
            .expect("Failed to send GET /jobs/:id after delete");

        assert_eq!(get_response.status(), 404);

        common_test::delete_jobs_via_api(&client, &job_ids).await;
        common_test::delete_user_via_api(&client, user.id).await;
    }

    // 🔍 Lookup & Search Endpoints

    #[tokio::test]
    async fn test_search_jobs() {
        let (client, user, jobs, job_ids) = common_test::build_client_and_user_with_n_jobs(5).await;
        let keyword = &jobs[2].job_name[0..3]; // partial match

        let response = client
            .get(format!(
                "{}/jobs/search?user_id={}&query={}",
                APP_HOST, user.id, keyword
            ))
            .send()
            .await
            .expect("Failed to send search request");

        assert_eq!(response.status(), 200);
        let matches: Vec<Job> = response.json().await.expect("Invalid search result");
        assert!(matches.iter().any(|job| job.job_name == jobs[2].job_name));

        common_test::delete_jobs_via_api(&client, &job_ids).await;
        common_test::delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_get_job_by_name() {
        let (client, user, jobs, job_ids) = common_test::build_client_and_user_with_n_jobs(3).await;
        let exact_name = &jobs[0].job_name;

        let response = client
            .get(format!(
                "{}/jobs/name/{}?name={}",
                APP_HOST, user.id, exact_name
            ))
            .send()
            .await
            .expect("Failed to send exact name request");

        assert_eq!(response.status(), 200);
        let matches: Vec<Job> = response.json().await.expect("Invalid name lookup");
        assert!(matches.iter().any(|job| job.job_name == *exact_name));

        common_test::delete_jobs_via_api(&client, &job_ids).await;
        common_test::delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_get_jobs_by_admin() {
        let (client, user, _jobs, job_ids) =
            common_test::build_client_and_user_with_n_jobs(4).await;

        let response = client
            .get(format!(
                "{}/jobs/by_admin?user_id={}&limit=10&offset=0",
                APP_HOST, user.id
            ))
            .send()
            .await
            .expect("Failed to send by_admin request");

        assert_eq!(response.status(), 200);
        let found: Vec<Job> = response.json().await.expect("Invalid by_admin response");
        assert!(found.len() >= 4);

        common_test::delete_jobs_via_api(&client, &job_ids).await;
        common_test::delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_get_jobs_by_state() {
        let (client, user, jobs, job_ids) = common_test::build_client_and_user_with_n_jobs(2).await;
        let job_id = job_ids[0];

        // Manually mark one job as Failed
        let updated_payload = serde_json::json!({
            "id": job_id,
            "user_id": user.id,
            "job_name": jobs[0].job_name,
            "image_url": "docker.io/library/alpine:latest",
            "image_format": "DockerRegistry",
            "docker_flags": null,
            "output_type": "Stdout",
            "output_paths": null,
            "schedule_type": "Once",
            "cron_expression": null,
            "notes": null,
            "state": "Failed",
            "created_at": jobs[0].created_at,
            "updated_at": Utc::now().naive_utc(),
        });

        let response = client
            .patch(format!("{}/jobs/{job_id}", common_test::APP_HOST))
            .json(&updated_payload)
            .send()
            .await
            .expect("Failed to send PATCH /jobs/:id");
        assert_eq!(response.status(), 200);

        let response = client
            .get(format!("{}/jobs/state/Failed", APP_HOST))
            .send()
            .await
            .expect("Failed to fetch by state");

        assert_eq!(response.status(), 200);
        let found: Vec<Job> = response.json().await.expect("Invalid response by state");
        assert!(found
            .iter()
            .any(|job| job.id == job_id && job.state == JobStateEnum::Failed));

        common_test::delete_jobs_via_api(&client, &job_ids).await;
        common_test::delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_get_recent_jobs() {
        let (client, user, _jobs, job_ids) =
            common_test::build_client_and_user_with_n_jobs(3).await;

        let response = client
            .get(format!("{}/jobs/recent?limit=5", APP_HOST))
            .send()
            .await
            .expect("Failed to get recent jobs");

        assert_eq!(response.status(), 200);
        let recent: Vec<Job> = response.json().await.expect("Invalid recent jobs response");
        assert!(!recent.is_empty());

        common_test::delete_jobs_via_api(&client, &job_ids).await;
        common_test::delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_get_failed_jobs() {
        let (client, user, jobs, job_ids) = common_test::build_client_and_user_with_n_jobs(2).await;
        let job_id = job_ids[1];

        // Manually mark one job as Failed
        let updated_payload = serde_json::json!({
            "id": job_id,
            "user_id": user.id,
            "job_name": jobs[0].job_name,
            "image_url": "docker.io/library/alpine:latest",
            "image_format": "DockerRegistry",
            "docker_flags": null,
            "output_type": "Stdout",
            "output_paths": null,
            "schedule_type": "Once",
            "cron_expression": null,
            "notes": null,
            "state": "Failed",
            "created_at": jobs[0].created_at,
            "updated_at": Utc::now().naive_utc(),
        });

        let response = client
            .patch(format!("{}/jobs/{job_id}", common_test::APP_HOST))
            .json(&updated_payload)
            .send()
            .await
            .expect("Failed to send PATCH /jobs/:id");
        assert_eq!(response.status(), 200);

        let response = client
            .get(format!("{}/jobs/failed?limit=10", APP_HOST))
            .send()
            .await
            .expect("Failed to get failed jobs");

        assert_eq!(response.status(), 200);
        let failed_jobs: Vec<Job> = response.json().await.expect("Invalid failed jobs response");
        assert!(!failed_jobs.is_empty());
        assert!(failed_jobs
            .iter()
            .any(|job| job.id == job_id && job.state == JobStateEnum::Failed));

        common_test::delete_jobs_via_api(&client, &job_ids).await;
        common_test::delete_user_via_api(&client, user.id).await;
    }

    // 🔄 State Transitions

    #[tokio::test]
    async fn test_mark_job_running() {
        let (client, user, jobs, job_ids) = common_test::build_client_and_user_with_n_jobs(1).await;
        let job_id = job_ids[0];
        let job = &jobs[0];

        let updated_payload = serde_json::json!({
            "id": job_id,
            "user_id": user.id,
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
            .patch(format!("{}/jobs/{job_id}/running", APP_HOST))
            .json(&updated_payload)
            .send()
            .await
            .expect("Failed to mark job running");

        assert_eq!(response.status(), 200);
        let job: Job = response.json().await.expect("Invalid response");
        assert_eq!(job.state, JobStateEnum::Running);

        common_test::delete_jobs_via_api(&client, &job_ids).await;
        common_test::delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_mark_job_succeeded() {
        let (client, user, jobs, job_ids) = common_test::build_client_and_user_with_n_jobs(1).await;
        let job_id = job_ids[0];
        let job = &jobs[0];

        let updated_payload = serde_json::json!({
            "id": job_id,
            "user_id": user.id,
            "job_name": job.job_name,
            "image_url": job.image_url,
            "image_format": job.image_format,
            "docker_flags": job.docker_flags,
            "output_type": job.output_type,
            "output_paths": job.output_paths,
            "schedule_type": job.schedule_type,
            "cron_expression": job.cron_expression,
            "notes": job.notes,
            "state": "Completed",
            "created_at": job.created_at,
            "updated_at": Utc::now().naive_utc(),
        });

        let response = client
            .patch(format!("{}/jobs/{job_id}", APP_HOST))
            .json(&updated_payload)
            .send()
            .await
            .expect("Failed to mark job complete");

        assert_eq!(response.status(), 200);
        let job: Job = response.json().await.expect("Invalid response");
        assert_eq!(job.state, JobStateEnum::Completed);

        common_test::delete_jobs_via_api(&client, &job_ids).await;
        common_test::delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_mark_job_failed() {
        let (client, user, jobs, job_ids) = common_test::build_client_and_user_with_n_jobs(1).await;
        let job_id = job_ids[0];
        let job = &jobs[0];

        let updated_payload = serde_json::json!({
            "id": job_id,
            "user_id": user.id,
            "job_name": job.job_name,
            "image_url": job.image_url,
            "image_format": job.image_format,
            "docker_flags": job.docker_flags,
            "output_type": job.output_type,
            "output_paths": job.output_paths,
            "schedule_type": job.schedule_type,
            "cron_expression": job.cron_expression,
            "notes": "Example failure reason",
            "state": "Failed",
            "created_at": job.created_at,
            "updated_at": Utc::now().naive_utc(),
        });

        let response = client
            .patch(format!("{}/jobs/{job_id}/failed", APP_HOST))
            .json(&updated_payload)
            .send()
            .await
            .expect("Failed to mark job failed");

        assert_eq!(response.status(), 200);
        let job: Job = response.json().await.expect("Invalid response");
        assert_eq!(job.state, JobStateEnum::Failed);

        common_test::delete_jobs_via_api(&client, &job_ids).await;
        common_test::delete_user_via_api(&client, user.id).await;
    }

    // ⏱️ Scheduling & Readiness

    #[tokio::test]
    async fn test_get_cron_jobs_due() {
        // not yet fully understood or implemented,
        // TODO:
    }

    #[tokio::test]
    async fn test_get_ready_one_time_jobs() {
        let (client, user, jobs, job_ids) = common_test::build_client_and_user_with_n_jobs(2).await;

        //mark both jobs ready
        let job_id = job_ids[0];
        let job = &jobs[0];
        let updated_payload = serde_json::json!({
            "id": job_id,
            "user_id": user.id,
            "job_name": job.job_name,
            "image_url": job.image_url,
            "image_format": job.image_format,
            "docker_flags": job.docker_flags,
            "output_type": job.output_type,
            "output_paths": job.output_paths,
            "schedule_type": "Once",
            "cron_expression": job.cron_expression,
            "notes": "Example failure reason",
            "state": "Queued",
            "created_at": job.created_at,
            "updated_at": Utc::now().naive_utc(),
        });

        let response = client
            .patch(format!("{}/jobs/{job_id}", APP_HOST))
            .json(&updated_payload)
            .send()
            .await
            .expect("Failed to mark job failed");

        // Second job
        let job_id = job_ids[1];
        let job = &jobs[1];
        let updated_payload = serde_json::json!({
            "id": job_id,
            "user_id": user.id,
            "job_name": job.job_name,
            "image_url": job.image_url,
            "image_format": job.image_format,
            "docker_flags": job.docker_flags,
            "output_type": job.output_type,
            "output_paths": job.output_paths,
            "schedule_type": "Once",
            "cron_expression": job.cron_expression,
            "notes": "Example failure reason",
            "state": "Queued",
            "created_at": job.created_at,
            "updated_at": Utc::now().naive_utc(),
        });

        let response = client
            .patch(format!("{}/jobs/{job_id}", APP_HOST))
            .json(&updated_payload)
            .send()
            .await
            .expect("Failed to mark job failed");

        let response = client
            .get(format!("{}/jobs/one-time-ready", APP_HOST))
            .send()
            .await
            .expect("Failed to get ready jobs");

        assert_eq!(response.status(), 200);
        let ready_jobs: Vec<Job> = response.json().await.expect("Invalid ready jobs response");
        assert!(ready_jobs.len() >= 2);
        assert!(!ready_jobs.is_empty());

        common_test::delete_jobs_via_api(&client, &job_ids).await;
        common_test::delete_user_via_api(&client, user.id).await;
    }

    // 📊 Aggregation & Stats

    #[tokio::test]
    async fn test_get_job_stats_by_admin() {
        let (client, user, jobs, job_ids) = common_test::build_client_and_user_with_n_jobs(3).await;

        let response = client
            .get(format!("{}/jobs/stats/admins", common_test::APP_HOST))
            .send()
            .await
            .expect("Failed to GET job stats");

        assert_eq!(response.status(), 200);
        let stats: Vec<(i32, i64)> = response.json().await.expect("Invalid stats format");

        // Ensure our admin's ID is present and job count matches
        let entry = stats.iter().find(|(admin_id, _)| *admin_id == user.id);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().1, 3);

        common_test::delete_jobs_via_api(&client, &job_ids).await;
        common_test::delete_user_via_api(&client, user.id).await;
    }

    // 🤝 Assignment & Worker Routing

    #[tokio::test]
    async fn test_get_active_jobs_for_worker() {
        let (client, user, jobs, job_ids) = common_test::build_client_and_user_with_n_jobs(2).await;
        let worker = common_test::create_worker_via_api(&client, user.id).await;

        common_test::assign_job_to_worker(&client, job_ids[0], worker.id).await;
        common_test::mark_job_running(&client, job_ids[0]).await;

        let response = client
            .get(format!(
                "{}/jobs/active/{}",
                common_test::APP_HOST,
                worker.id
            ))
            .send()
            .await
            .expect("Failed to GET active jobs for worker");

        assert_eq!(response.status(), 200);
        let jobs: Vec<Job> = response.json().await.expect("Invalid jobs response");
        assert!(jobs
            .iter()
            .any(|j| j.id == job_ids[0] && j.state == JobStateEnum::Running));

        common_test::delete_jobs_via_api(&client, &job_ids).await;
        common_test::delete_worker_via_api(&client, worker.id).await;
        common_test::delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_get_assigned_jobs_for_worker() {
        let (client, user, jobs, job_ids) = common_test::build_client_and_user_with_n_jobs(2).await;
        let worker = common_test::create_worker_via_api(&client, user.id).await;

        common_test::assign_job_to_worker(&client, job_ids[1], worker.id).await;

        let response = client
            .get(format!(
                "{}/jobs/assigned/{}",
                common_test::APP_HOST,
                worker.id
            ))
            .send()
            .await
            .expect("Failed to GET assigned jobs for worker");

        assert_eq!(response.status(), 200);
        let jobs: Vec<Job> = response.json().await.expect("Invalid jobs response");
        assert!(jobs.iter().any(|j| j.id == job_ids[1]));

        common_test::delete_jobs_via_api(&client, &job_ids).await;
        common_test::delete_worker_via_api(&client, worker.id).await;
        common_test::delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_get_unassigned_jobs() {
        let (client, user, jobs, job_ids) = common_test::build_client_and_user_with_n_jobs(2).await;
        let worker = common_test::create_worker_via_api(&client, user.id).await;

        // Assign one job, leave the other unassigned
        common_test::assign_job_to_worker(&client, job_ids[0], worker.id).await;

        let response = client
            .get(format!("{}/jobs/unassigned", common_test::APP_HOST))
            .send()
            .await
            .expect("Failed to GET unassigned jobs");

        assert_eq!(response.status(), 200);
        let jobs: Vec<Job> = response.json().await.expect("Invalid jobs response");
        assert!(!jobs.is_empty());

        common_test::delete_jobs_via_api(&client, &job_ids).await;
        common_test::delete_worker_via_api(&client, worker.id).await;
        common_test::delete_user_via_api(&client, user.id).await;
    }
}
