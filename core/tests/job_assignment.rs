/* ===================== ⚙️ JobAssignment API Overview =====================

== 🛠️ CRUD ==
• POST   /assignments                    → Create new assignment (NewJobAssignment) → 201 Created (JobAssignment)
• GET    /assignments/:id               → Fetch assignment by ID → 200 OK (JobAssignment)
• DELETE /assignments/:id               → Delete assignment by ID → 204 No Content

== 🔍 Lookup & Search ==
• GET /assignments/by_job/:job_id                  → Assignments by Job ID → 200 OK (Vec<JobAssignment>)
• GET /assignments/by_worker/:worker_id            → Assignments by Worker ID → 200 OK (Vec<JobAssignment>)
• GET /assignments/lookup/:job_id/:worker_id       → Assignment by Job + Worker → 200 OK (Option<JobAssignment>)
• GET /assignments/by_worker/range?worker_id&start&end
                                                  → Assignments in time range for worker → 200 OK (Vec<JobAssignment>)
• GET /assignments/active                          → Currently active assignments → 200 OK (Vec<JobAssignment>)

== 🔄 State Transitions ==
• PATCH /assignments/:id/started   → Mark assignment as started (NaiveDateTime) → 200 OK (JobAssignment)
• PATCH /assignments/:id/finished  → Mark assignment as finished (NaiveDateTime) → 200 OK (JobAssignment)
======================================================================== */
pub mod common;

#[cfg(test)]
mod job_assignment_api_tests {
    use crate::common::{
        assign_job_to_worker, build_client_and_user_with_n_jobs, create_worker_via_api,
        delete_user_via_api, delete_worker_via_api, APP_HOST,
    };
    use chrono::Utc;
    use reqwest::Client;
    use serde_json::json;
    use uuid::Uuid;
    // 🚀 CRUD Endpoints

    #[tokio::test]
    async fn test_create_job_assignment() {
        let (client, user, jobs, job_ids) = build_client_and_user_with_n_jobs(1).await;
        let job = &jobs[0];
        let worker = create_worker_via_api(&client, user.id).await;

        let assignment = assign_job_to_worker(&client, job.id, worker.id).await;

        assert_eq!(assignment.job_id, job.id);
        assert_eq!(assignment.worker_id, worker.id);
        assert!(assignment.assigned_at <= Utc::now().naive_utc());

        // cleanup: job -> worker -> user
        client
            .delete(&format!("{}/jobs/{}", APP_HOST, job.id))
            .send()
            .await
            .expect("Failed to delete job");

        delete_worker_via_api(&client, worker.id).await;

        //make sure that job assignment does not exist
        let lookup_url = format!("{}/assignments/lookup/{}/{}", APP_HOST, job.id, worker.id);
        let res = client.get(&lookup_url).send().await.expect("GET failed");

        assert_eq!(
            res.status().as_u16(),
            404,
            "Expected assignment to be gone after worker deletion"
        );

        delete_user_via_api(&client, user.id).await;
    }

    #[test]
    fn test_get_job_assignment_by_id() {
        // Test logic goes here
    }

    #[test]
    fn test_delete_job_assignment() {
        // Test logic goes here
    }

    // 🔍 Lookup & Search Endpoints

    #[test]
    fn test_get_assignments_by_job_id() {
        // Test logic goes here
    }

    #[test]
    fn test_get_assignments_by_worker_id() {
        // Test logic goes here
    }

    #[test]
    fn test_get_assignment_by_job_and_worker() {
        // Test logic goes here
    }

    #[test]
    fn test_get_assignments_for_worker_in_time_range() {
        // Test logic goes here
    }

    #[test]
    fn test_get_active_job_assignments() {
        // Test logic goes here
    }

    // 🔄 State Transitions

    #[test]
    fn test_mark_job_assignment_started() {
        // Test logic goes here
    }

    #[test]
    fn test_mark_job_assignment_finished() {
        // Test logic goes here
    }
}
