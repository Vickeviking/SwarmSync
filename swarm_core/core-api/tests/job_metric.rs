/* ===================== ⚙️ JobMetric API Overview =====================

== 🛠️ CRUD ==
• POST   /metrics                             → Create new metric (NewJobMetric) → 201 Created (JobMetric)
• GET    /metrics/:id                         → Fetch metric by ID             → 200 OK (JobMetric)
• DELETE /metrics/:id                         → Delete metric by ID            → 204 No Content

== 🔍 Lookup & Search ==
• GET    /metrics/by_job/:job_id              → Metrics by Job ID              → 200 OK (Vec<JobMetric>)
• GET    /metrics/by_worker/:worker_id        → Metrics by Worker ID           → 200 OK (Vec<JobMetric>)
• GET    /metrics/recent/:job_id              → Most recent metric for Job     → 200 OK (Option<JobMetric>)
• GET    /metrics/worker_stream/:worker_id    → Worker metric stream           → 200 OK (Vec<JobMetric>)

======================================================================== */
pub mod common_test;

#[cfg(test)]
mod job_metric_api_tests {
    use crate::common_test::{
        assign_job_to_worker, build_client_and_user_with_n_jobs, create_metric_via_api,
        create_worker_via_api, delete_job_via_api, delete_jobs_via_api, delete_user_via_api,
        delete_worker_via_api, get_ndt_now, mark_assignment_finished_via_api, APP_HOST,
    };
    use chrono::Utc;
    use common::database::models::job::JobMetric;
    use rocket::serde::json::json;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_create_job_metric() {
        let (client, user, jobs, _) = build_client_and_user_with_n_jobs(1).await;
        let job = &jobs[0];
        let worker = create_worker_via_api(&client, user.id).await;
        assign_job_to_worker(&client, job.id, worker.id).await;

        let payload = json!({
            "job_id": job.id,
            "worker_id": worker.id,
            "duration_sec": 42,
            "cpu_usage_pct": 63.5,
            "mem_usage_mb": 128.0,
            "exit_code": 0
        });

        let url = format!("{}/metrics", APP_HOST);
        let res = client.post(&url).json(&payload).send().await.unwrap();
        assert_eq!(res.status().as_u16(), 201);

        let metric: JobMetric = res.json().await.unwrap();
        assert_eq!(metric.job_id, job.id);
        assert_eq!(metric.worker_id, worker.id);
        assert_eq!(metric.duration_sec, Some(42));
        assert_eq!(metric.exit_code, Some(0));

        delete_job_via_api(&client, job.id).await;
        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_metric_upsert_updates_existing_row() {
        let (client, user, jobs, _) = build_client_and_user_with_n_jobs(1).await;
        let job = &jobs[0];
        let worker = create_worker_via_api(&client, user.id).await;
        assign_job_to_worker(&client, job.id, worker.id).await;

        // First metric creation
        let payload_1 = json!({
            "job_id": job.id,
            "worker_id": worker.id,
            "duration_sec": 10,
            "cpu_usage_pct": 50.0,
            "mem_usage_mb": 100.0,
            "exit_code": 0
        });

        let create_url = format!("{}/metrics", APP_HOST);
        let res_1 = client
            .post(&create_url)
            .json(&payload_1)
            .send()
            .await
            .unwrap();
        assert_eq!(res_1.status().as_u16(), 201);
        let metric_1: JobMetric = res_1.json().await.unwrap();

        // Second metric creation for the same (job_id, worker_id) — should overwrite
        let payload_2 = json!({
            "job_id": job.id,
            "worker_id": worker.id,
            "duration_sec": 99,
            "cpu_usage_pct": 88.8,
            "mem_usage_mb": 256.0,
            "exit_code": 137
        });

        let res_2 = client
            .post(&create_url)
            .json(&payload_2)
            .send()
            .await
            .unwrap();
        assert_eq!(res_2.status().as_u16(), 201);
        let metric_2: JobMetric = res_2.json().await.unwrap();

        // Should be the same row ID (i.e., it was updated)
        assert_eq!(metric_1.id, metric_2.id);

        // Check updated values are reflected
        assert_eq!(metric_2.duration_sec, Some(99));
        assert_eq!(metric_2.cpu_usage_pct, Some(88.8));
        assert_eq!(metric_2.exit_code, Some(137));

        // Clean up
        delete_job_via_api(&client, job.id).await;
        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_get_job_metric_by_id() {
        let (client, user, jobs, _) = build_client_and_user_with_n_jobs(1).await;
        let job = &jobs[0];
        let worker = create_worker_via_api(&client, user.id).await;
        assign_job_to_worker(&client, job.id, worker.id).await;

        let create_url = format!("{}/metrics", APP_HOST);
        let payload = json!({
            "job_id": job.id,
            "worker_id": worker.id,
            "duration_sec": 55,
            "exit_code": 1
        });

        let created: JobMetric = client
            .post(&create_url)
            .json(&payload)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let get_url = format!("{}/metrics/{}", APP_HOST, created.id);
        let fetched: JobMetric = client
            .get(&get_url)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.exit_code, Some(1));

        delete_job_via_api(&client, job.id).await;
        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_delete_job_metric() {
        let (client, user, jobs, _) = build_client_and_user_with_n_jobs(1).await;
        let job = &jobs[0];
        let worker = create_worker_via_api(&client, user.id).await;
        assign_job_to_worker(&client, job.id, worker.id).await;

        let payload = json!({
            "job_id": job.id,
            "worker_id": worker.id,
            "exit_code": 0
        });

        let res = client
            .post(&format!("{}/metrics", APP_HOST))
            .json(&payload)
            .send()
            .await
            .unwrap();
        let metric: JobMetric = res.json().await.unwrap();

        let delete_url = format!("{}/metrics/{}", APP_HOST, metric.id);
        let del_res = client.delete(&delete_url).send().await.unwrap();
        assert_eq!(del_res.status().as_u16(), 204);

        delete_job_via_api(&client, job.id).await;
        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_get_metrics_by_job_id() {
        let (client, user, jobs, _) = build_client_and_user_with_n_jobs(1).await;
        let job = &jobs[0];
        let worker = create_worker_via_api(&client, user.id).await;
        assign_job_to_worker(&client, job.id, worker.id).await;
        create_metric_via_api(&client, job.id, worker.id, 4, 4.0, 4.0, 4).await;

        let res = client
            .get(&format!("{}/metrics/by_job/{}", APP_HOST, job.id))
            .send()
            .await
            .unwrap();
        let metrics: JobMetric = res.json().await.unwrap();

        delete_job_via_api(&client, job.id).await;
        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_get_metrics_by_worker_id() {
        let (client, user, jobs, _) = build_client_and_user_with_n_jobs(2).await;
        let job1 = &jobs[0];
        let job2 = &jobs[1];
        let worker = create_worker_via_api(&client, user.id).await;

        //worker has 2 jobs assigned now
        assign_job_to_worker(&client, job1.id, worker.id).await;
        assign_job_to_worker(&client, job2.id, worker.id).await;

        //now create metrics for these 2 jobs

        create_metric_via_api(&client, job1.id, worker.id, 4, 4.0, 4.0, 4).await;
        create_metric_via_api(&client, job2.id, worker.id, 4, 4.0, 4.0, 4).await;

        let res = client
            .get(&format!("{}/metrics/by_worker/{}", APP_HOST, worker.id))
            .send()
            .await
            .unwrap();
        let metrics: Vec<JobMetric> = res.json().await.unwrap();
        assert!(metrics.len() >= 2); // Can be more if other tests run in parallel

        delete_jobs_via_api(&client, &vec![job1.id, job2.id]).await;
        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_get_most_recent_metric_for_job() {
        let (client, user, jobs, _) = build_client_and_user_with_n_jobs(1).await;
        let job = &jobs[0];
        let worker = create_worker_via_api(&client, user.id).await;
        assign_job_to_worker(&client, job.id, worker.id).await;

        for _ in 0..3 {
            let payload = json!({
                "job_id": job.id,
                "worker_id": worker.id,
                "exit_code": 0
            });
            client
                .post(&format!("{}/metrics", APP_HOST))
                .json(&payload)
                .send()
                .await
                .unwrap();
            sleep(Duration::from_millis(10)).await;
        }

        let res = client
            .get(&format!("{}/metrics/recent/{}", APP_HOST, job.id))
            .send()
            .await
            .unwrap();
        let recent: Option<JobMetric> = res.json().await.unwrap();
        assert!(recent.is_some());

        delete_job_via_api(&client, job.id).await;
        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }
}
