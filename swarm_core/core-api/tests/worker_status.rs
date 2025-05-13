pub mod common_test;
#[cfg(test)]
mod worker_status_api_tests {

    use crate::common_test::{
        build_client_and_user_with_n_jobs, build_client_with_logged_in_admin,
        create_worker_status_via_api, create_worker_via_api, delete_jobs_via_api,
        delete_user_via_api, delete_worker_via_api, get_ndt_now, APP_HOST,
    };
    use common::database::models::worker::WorkerStatus;
    use reqwest::StatusCode;
    use rocket::serde::json::json;

    // 🚀 CRUD Endpoints

    #[tokio::test]
    async fn test_create_worker_status() {
        let (client, user) = build_client_with_logged_in_admin().await;
        let worker = create_worker_via_api(&client, user.id).await;

        let created_status = create_worker_status_via_api(&client, worker.id, None).await;

        assert_eq!(
            created_status.worker_id, worker.id,
            "Worker ID should match"
        );

        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_get_worker_status_by_id() {
        let (client, user) = build_client_with_logged_in_admin().await;
        let worker = create_worker_via_api(&client, user.id).await;
        let created_status = create_worker_status_via_api(&client, worker.id, None).await;

        let res = client
            .get(&format!("{}/worker-status/{}", APP_HOST, created_status.id))
            .send()
            .await
            .expect("Failed to fetch worker status");

        assert_eq!(res.status(), StatusCode::OK);
        let fetched: WorkerStatus = res.json().await.expect("Failed to parse response");
        assert_eq!(fetched.id, created_status.id);

        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_delete_worker_status_by_id() {
        let (client, user) = build_client_with_logged_in_admin().await;
        let worker = create_worker_via_api(&client, user.id).await;
        let created_status = create_worker_status_via_api(&client, worker.id, None).await;

        let res = client
            .delete(&format!("{}/worker-status/{}", APP_HOST, created_status.id))
            .send()
            .await
            .expect("Failed to delete worker status");

        assert_eq!(res.status(), StatusCode::NO_CONTENT);

        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_find_status_by_worker_id() {
        let (client, user) = build_client_with_logged_in_admin().await;
        let worker = create_worker_via_api(&client, user.id).await;
        let created_status = create_worker_status_via_api(&client, worker.id, None).await;

        let res = client
            .get(&format!("{}/worker-status/worker/{}", APP_HOST, worker.id))
            .send()
            .await
            .expect("Failed to lookup status by worker ID");

        assert_eq!(res.status(), StatusCode::OK);
        let fetched: Option<WorkerStatus> = res.json().await.expect("Failed to parse response");
        assert_eq!(fetched.unwrap().id, created_status.id);

        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    // 🔄 State Update Endpoints

    #[tokio::test]
    async fn test_update_worker_status() {
        let (client, user) = build_client_with_logged_in_admin().await;
        let worker = create_worker_via_api(&client, user.id).await;
        let status = create_worker_status_via_api(&client, worker.id, None).await;

        let res = client
            .put(&format!("{}/worker-status/{}/status", APP_HOST, status.id))
            .json(&json!("Idle"))
            .send()
            .await
            .expect("Failed to update worker status");

        assert_eq!(res.status(), StatusCode::OK);
        let updated: WorkerStatus = res.json().await.expect("Failed to parse response");
        assert_eq!(updated.status.to_string(), "Idle");

        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }
    #[tokio::test]
    async fn test_update_last_heartbeat_timestamp() {
        let (client, user) = build_client_with_logged_in_admin().await;
        let worker = create_worker_via_api(&client, user.id).await;
        let status = create_worker_status_via_api(&client, worker.id, None).await;

        // No need to send a JSON payload now
        let res = client
            .put(&format!(
                "{}/worker-status/{}/last-heartbeat",
                APP_HOST, status.id
            ))
            .send()
            .await
            .expect("Failed to update heartbeat");

        assert_eq!(res.status(), StatusCode::OK);

        let updated: WorkerStatus = res.json().await.expect("Failed to parse response");

        // Optional: Check that heartbeat is *after* test start time
        let now = get_ndt_now();
        assert!(updated.last_heartbeat.unwrap() <= now);

        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_update_active_job_id() {
        let (client, user, jobs, job_ids) = build_client_and_user_with_n_jobs(1).await;
        let job_id = jobs[0].id;
        let worker = create_worker_via_api(&client, user.id).await;
        let status = create_worker_status_via_api(&client, worker.id, None).await;

        let res = client
            .put(&format!(
                "{}/worker-status/{}/active-job-id",
                APP_HOST, status.id
            ))
            .json(&json!({ "active_job_id": job_id }))
            .send()
            .await
            .expect("Failed to update active job ID");

        assert_eq!(res.status(), StatusCode::OK);
        let updated: WorkerStatus = res.json().await.expect("Failed to parse response");
        assert_eq!(updated.active_job_id, Some(job_id));

        delete_jobs_via_api(&client, &job_ids).await;
        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_update_uptime_in_seconds() {
        let (client, user) = build_client_with_logged_in_admin().await;
        let worker = create_worker_via_api(&client, user.id).await;
        let status = create_worker_status_via_api(&client, worker.id, None).await;

        let res = client
            .put(&format!("{}/worker-status/{}/uptime", APP_HOST, status.id))
            .json(&json!({ "uptime": 7200 }))
            .send()
            .await
            .expect("Failed to update uptime");

        assert_eq!(res.status(), StatusCode::OK);
        let updated: WorkerStatus = res.json().await.expect("Failed to parse response");
        assert_eq!(updated.uptime_sec, Some(7200));

        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_update_load_avg() {
        let (client, user) = build_client_with_logged_in_admin().await;
        let worker = create_worker_via_api(&client, user.id).await;
        let status = create_worker_status_via_api(&client, worker.id, None).await;

        let new_load = vec![Some(1.0), Some(0.9), Some(0.8)];

        let res = client
            .put(&format!(
                "{}/worker-status/{}/load-avg",
                APP_HOST, status.id
            ))
            .json(&json!({ "load_avg": new_load }))
            .send()
            .await
            .expect("Failed to update load average");

        assert_eq!(res.status(), StatusCode::OK);
        let updated: WorkerStatus = res.json().await.expect("Failed to parse response");
        assert_eq!(updated.load_avg.unwrap(), new_load);

        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_update_last_error_message() {
        let (client, user) = build_client_with_logged_in_admin().await;
        let worker = create_worker_via_api(&client, user.id).await;
        let status = create_worker_status_via_api(&client, worker.id, None).await;

        let res = client
            .put(&format!(
                "{}/worker-status/{}/last-error",
                APP_HOST, status.id
            ))
            .json(&json!({ "last_error": "Out of memory" }))
            .send()
            .await
            .expect("Failed to update last error");

        assert_eq!(res.status(), StatusCode::OK);
        let updated: WorkerStatus = res.json().await.expect("Failed to parse response");
        assert_eq!(updated.last_error.as_deref(), Some("Out of memory"));

        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }
}
