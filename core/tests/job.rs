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

pub mod common;

use rocket::serde::json::json;
use swarmsync_core::database::models::job::NewJob;
use swarmsync_core::shared::enums::image_format::ImageFormatEnum;
use swarmsync_core::shared::enums::job::JobStateEnum;
use swarmsync_core::shared::enums::output::OutputTypeEnum;
use swarmsync_core::shared::enums::schedule::ScheduleTypeEnum;

#[cfg(test)]
mod job_api_tests {
    use reqwest::StatusCode;
    use swarmsync_core::database::models::job::Job;

    use super::*;

    #[tokio::test]
    async fn test_create_job() {
        // Step 1: Build the authorized client with a logged-in admin user
        let (client, user) = common::build_client_with_logged_in_admin().await;

        // Step 2: Prepare the job payload
        let new_job = NewJob {
            user_id: user.id,
            job_name: common::generate_unique_job_name(),
            image_url: String::from("docker.io/library/alpine:latest"),
            image_format: ImageFormatEnum::DockerRegistry,
            docker_flags: None,
            output_type: OutputTypeEnum::Stdout,
            output_paths: None,
            schedule_type: ScheduleTypeEnum::Once,
            cron_expression: None,
            notes: None,
            state: JobStateEnum::Queued,
        };

        // Step 3: Check debug output for the state field
        dbg!(&new_job.state);

        // Step 4: Prepare and log the outgoing JSON
        let payload_json = json!({
            "user_id": new_job.user_id,
            "job_name": new_job.job_name,
            "image_url": new_job.image_url,
            "image_format": new_job.image_format.to_string(),
            "docker_flags": new_job.docker_flags,
            "output_type": new_job.output_type.to_string(),
            "output_paths": new_job.output_paths,
            "schedule_type": new_job.schedule_type.to_string(),
            "cron_expression": new_job.cron_expression,
            "notes": new_job.notes,
            "state": new_job.state.to_string(),
        });

        println!(
            "📦 Serialized JSON payload:\n{}",
            serde_json::to_string_pretty(&payload_json).unwrap()
        );

        // Step 5: Ensure the JSON is actually deserializable by NewJob
        match serde_json::from_value::<NewJob>(payload_json.clone()) {
            Ok(validated) => println!(
                "✅ JSON successfully deserializes to NewJob:\n{:#?}",
                validated
            ),
            Err(e) => panic!(
                "❌ JSON failed to deserialize into NewJob:\n{}\nPayload: {}",
                e, payload_json
            ),
        }

        // Step 6: Send the request
        let response = client
            .post(format!("{}/jobs", common::APP_HOST))
            .json(&payload_json)
            .send()
            .await
            .expect("❌ Failed to send create job request");

        // Step 7: Check status
        assert_eq!(response.status(), StatusCode::CREATED);

        // Step 8: Parse the returned job
        let body = response
            .text()
            .await
            .expect("❌ Failed to read response body");

        let created_job: Job = serde_json::from_str(&body).unwrap_or_else(|e| {
            panic!(
                "❌ Failed to parse Job from response JSON:\nError: {}\nBody:\n{}",
                e, body
            )
        });

        // Step 9: Assert the job content
        assert_eq!(created_job.user_id, new_job.user_id);
        assert_eq!(created_job.job_name, new_job.job_name);
        assert_eq!(created_job.image_url, new_job.image_url);
        assert_eq!(created_job.image_format, new_job.image_format);
        assert_eq!(created_job.state, JobStateEnum::Queued);
    }

    #[test]
    fn test_get_job_by_id() {
        // Test logic goes here
    }

    #[test]
    fn test_update_job() {
        // Test logic goes here
    }

    #[test]
    fn test_delete_job() {
        // Test logic goes here
    }

    // 🔍 Lookup & Search Endpoints

    #[test]
    fn test_search_jobs() {
        // Test logic goes here
    }

    #[test]
    fn test_get_job_by_name() {
        // Test logic goes here
    }

    #[test]
    fn test_get_jobs_by_admin() {
        // Test logic goes here
    }

    #[test]
    fn test_get_jobs_by_state() {
        // Test logic goes here
    }

    #[test]
    fn test_get_recent_jobs() {
        // Test logic goes here
    }

    #[test]
    fn test_get_failed_jobs() {
        // Test logic goes here
    }

    // 🔄 State Transitions

    #[test]
    fn test_mark_job_running() {
        // Test logic goes here
    }

    #[test]
    fn test_mark_job_succeeded() {
        // Test logic goes here
    }

    #[test]
    fn test_mark_job_failed() {
        // Test logic goes here
    }

    // ⏱️ Scheduling & Readiness

    #[test]
    fn test_get_scheduled_jobs() {
        // Test logic goes here
    }

    #[test]
    fn test_get_cron_jobs_due() {
        // Test logic goes here
    }

    #[test]
    fn test_get_ready_jobs() {
        // Test logic goes here
    }

    // 📊 Aggregation & Stats

    #[test]
    fn test_get_job_stats_by_admin() {
        // Test logic goes here
    }

    // 🤝 Assignment & Worker Routing

    #[test]
    fn test_get_active_jobs_for_worker() {
        // Test logic goes here
    }

    #[test]
    fn test_get_assigned_jobs_for_worker() {
        // Test logic goes here
    }

    #[test]
    fn test_get_unassigned_jobs() {
        // Test logic goes here
    }
}
