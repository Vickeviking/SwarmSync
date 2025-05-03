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
pub mod common_test;

#[cfg(test)]
mod job_assignment_api_tests {
    use crate::common_test::{
        assign_job_to_worker, build_client_and_user_with_n_jobs, create_worker_via_api,
        delete_job_via_api, delete_jobs_via_api, delete_user_via_api, delete_worker_via_api,
        get_ndt_now, mark_assignment_finished_via_api, APP_HOST,
    };
    use chrono::Utc;
    use common::database::models::job::JobAssignment;
    use rocket::serde::json::json;
    use tokio::time::{sleep, Duration};

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
        sleep(Duration::from_secs(1)).await; // Allow time for cascade deletion

        // Check if job assignment has been deleted using the ID
        let assignment_id = assignment.id; // Get the assignment ID

        let lookup_url = format!("{}/assignments/{}", APP_HOST, assignment_id);
        let res = client.get(&lookup_url).send().await.expect("GET failed");

        assert_eq!(
            res.status().as_u16(),
            404,
            "Expected assignment to be gone after worker deletion"
        );

        delete_user_via_api(&client, user.id).await;
    }
    #[tokio::test]
    async fn test_get_job_assignment_by_id() {
        let (client, user, jobs, _) = build_client_and_user_with_n_jobs(1).await;
        let job = &jobs[0];
        let worker = create_worker_via_api(&client, user.id).await;

        let assignment = assign_job_to_worker(&client, job.id, worker.id).await;
        let url = format!("{}/assignments/{}", APP_HOST, assignment.id);
        let res = client.get(&url).send().await.expect("Fetch by ID failed");
        assert_eq!(res.status().as_u16(), 200);
        let fetched = res.json::<JobAssignment>().await.expect("Deser failed");

        assert_eq!(fetched.id, assignment.id);
        assert_eq!(fetched.job_id, assignment.job_id);
        assert_eq!(fetched.worker_id, assignment.worker_id);

        // cleanup
        delete_job_via_api(&client, job.id).await;
        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_delete_job_assignment() {
        let (client, user, jobs, _) = build_client_and_user_with_n_jobs(1).await;
        let job = &jobs[0];
        let worker = create_worker_via_api(&client, user.id).await;

        let assignment = assign_job_to_worker(&client, job.id, worker.id).await;
        let url = format!("{}/assignments/{}", APP_HOST, assignment.id);
        let del = client.delete(&url).send().await.expect("DEL failed");
        assert_eq!(del.status().as_u16(), 204);

        // verify gone
        let res = client.get(&url).send().await.expect("GET failed");
        assert_eq!(res.status().as_u16(), 404);

        // cleanup
        delete_job_via_api(&client, job.id).await;
        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    // 🔍 Lookup & Search Endpoints

    #[tokio::test]
    async fn test_get_assignments_by_job_id() {
        let (client, user, jobs, _) = build_client_and_user_with_n_jobs(2).await;
        let job1 = &jobs[0];
        let job2 = &jobs[1];
        let worker1 = create_worker_via_api(&client, user.id).await;
        let worker2 = create_worker_via_api(&client, user.id).await;

        let a1 = assign_job_to_worker(&client, job1.id, worker1.id).await;
        let a2 = assign_job_to_worker(&client, job1.id, worker2.id).await;
        let _ = assign_job_to_worker(&client, job2.id, worker1.id).await;

        let url = format!("{}/assignments/by_job/{}", APP_HOST, job1.id);
        let res = client.get(&url).send().await.expect("GET by job failed");
        assert_eq!(res.status().as_u16(), 200);
        let list = res
            .json::<Vec<JobAssignment>>()
            .await
            .expect("Deser failed");

        let ids: Vec<i32> = list.iter().map(|x| x.id).collect();
        assert!(ids.contains(&a1.id) && ids.contains(&a2.id));

        // cleanup
        delete_job_via_api(&client, job1.id).await;
        delete_job_via_api(&client, job2.id).await;
        delete_worker_via_api(&client, worker1.id).await;
        delete_worker_via_api(&client, worker2.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_get_assignments_by_worker_id() {
        let (client, user, jobs, _) = build_client_and_user_with_n_jobs(2).await;
        let worker = create_worker_via_api(&client, user.id).await;
        let j1 = &jobs[0];
        let j2 = &jobs[1];

        let a1 = assign_job_to_worker(&client, j1.id, worker.id).await;
        let a2 = assign_job_to_worker(&client, j2.id, worker.id).await;

        let url = format!("{}/assignments/by_worker/{}", APP_HOST, worker.id);
        let res = client.get(&url).send().await.expect("GET by worker failed");
        assert_eq!(res.status().as_u16(), 200);
        let list = res
            .json::<Vec<JobAssignment>>()
            .await
            .expect("Deser failed");

        let ids: Vec<i32> = list.iter().map(|x| x.id).collect();
        assert!(ids.contains(&a1.id) && ids.contains(&a2.id));

        // cleanup
        for job in jobs {
            delete_job_via_api(&client, job.id).await;
        }
        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_get_assignment_by_job_and_worker() {
        let (client, user, jobs, _) = build_client_and_user_with_n_jobs(1).await;
        let job = &jobs[0];
        let worker = create_worker_via_api(&client, user.id).await;

        let assignment = assign_job_to_worker(&client, job.id, worker.id).await;

        let url = format!("{}/assignments/lookup/{}/{}", APP_HOST, job.id, worker.id);
        let res = client.get(&url).send().await.expect("Lookup failed");
        assert_eq!(res.status().as_u16(), 200);
        let fetched: Option<JobAssignment> = res.json().await.expect("Deser failed");
        let fetched = fetched.expect("Expected Some assignment");
        assert_eq!(fetched.id, assignment.id);

        // cleanup
        delete_job_via_api(&client, job.id).await;
        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_get_assignments_for_worker_in_time_range() {
        let (client, user, jobs, _) = build_client_and_user_with_n_jobs(1).await;
        let job = &jobs[0];
        let worker = create_worker_via_api(&client, user.id).await;

        // create two assignments at different times
        let a1 = assign_job_to_worker(&client, job.id, worker.id).await;
        sleep(Duration::from_secs(1)).await;
        let a2 = assign_job_to_worker(&client, job.id, worker.id).await;

        // time range from before a1 to after a2
        let start = Utc::now().naive_utc() - chrono::Duration::seconds(5);
        let end = Utc::now().naive_utc() + chrono::Duration::seconds(5);
        let url = format!(
            "{}/assignments/by_worker/range?worker_id={}&start={}&end={}",
            APP_HOST,
            worker.id,
            start.to_string(),
            end.to_string(),
        );
        let res = client.get(&url).send().await.expect("Range lookup failed");
        assert_eq!(res.status().as_u16(), 200);
        let list: Vec<JobAssignment> = res.json().await.expect("Deser failed");
        let ids: Vec<i32> = list.iter().map(|x| x.id).collect();
        assert!(ids.contains(&a1.id) && ids.contains(&a2.id));

        // cleanup
        delete_job_via_api(&client, job.id).await;
        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_get_active_job_assignments() {
        let (client, user, jobs, _) = build_client_and_user_with_n_jobs(2).await;
        let job1 = &jobs[0];
        let job2 = &jobs[1];
        let worker = create_worker_via_api(&client, user.id).await;

        let a1: JobAssignment = assign_job_to_worker(&client, job1.id, worker.id).await;
        let a2: JobAssignment = assign_job_to_worker(&client, job2.id, worker.id).await;

        // mark a1 finished
        mark_assignment_finished_via_api(&client, a1.id, get_ndt_now()).await;

        let url = format!("{}/assignments/active", APP_HOST);
        let res = client.get(&url).send().await.expect("Active lookup failed");
        assert_eq!(res.status().as_u16(), 200);
        let list: Vec<JobAssignment> = res.json().await.expect("Deser failed");
        let ids: Vec<i32> = list.iter().map(|x| x.id).collect();
        assert!(ids.contains(&a2.id) && !ids.contains(&a1.id));

        // cleanup
        delete_jobs_via_api(&client, &vec![job1.id, job2.id]).await;
        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }
    // 🔄 State Transitions

    #[tokio::test]
    async fn test_mark_job_assignment_started() {
        let (client, user, jobs, _) = build_client_and_user_with_n_jobs(1).await;
        let job = &jobs[0];
        let worker = create_worker_via_api(&client, user.id).await;

        let assignment = assign_job_to_worker(&client, job.id, worker.id).await;

        let start_ts = Utc::now().naive_utc();
        let payload = json!({ "started_at": start_ts.to_string() }); // Ensure it's a string

        println!("Payload: {}", payload); // Debug: Print the payload to ensure it's correct

        let url = format!("{}/assignments/{}/started", APP_HOST, assignment.id);

        let res = client
            .patch(&url)
            .json(&payload)
            .send()
            .await
            .expect("Patch start failed");

        // Extract status first (does not consume the response)
        let status = res.status().as_u16();
        println!("Response Status: {}", status); // Debugging the status code

        // Clone the response body for later use
        let body = res.text().await.expect("Failed to read response body");

        if status != 200 {
            println!("Response Body: {}", body);
        }
        assert!(status == 200);
        // If the status isn't 200, print the response body to understand the error

        // If status is 200, attempt to deserialize the JSON body
        let updated: JobAssignment = serde_json::from_str(&body).expect("Deserialization failed");

        // Truncate both timestamps to milliseconds before comparison
        let truncated_start_ts = start_ts.and_utc().timestamp_millis().to_string(); // Convert to milliseconds as string
        let truncated_updated = updated
            .started_at
            .unwrap()
            .and_utc()
            .timestamp_millis()
            .to_string(); // Convert to milliseconds as string

        assert_eq!(truncated_updated, truncated_start_ts);

        // Cleanup
        delete_job_via_api(&client, job.id).await;
        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_mark_job_assignment_finished() {
        let (client, user, jobs, _) = build_client_and_user_with_n_jobs(1).await;
        let job = &jobs[0];
        let worker = create_worker_via_api(&client, user.id).await;

        let assignment = assign_job_to_worker(&client, job.id, worker.id).await;

        let finish_ts = Utc::now().naive_utc();

        // Send the timestamp in a JSON object with the field "finished_at"
        let payload = json!({ "finished_at": finish_ts.to_string() });

        let url = format!("{}/assignments/{}/finished", APP_HOST, assignment.id);
        let res = client
            .patch(&url)
            .json(&payload) // Send JSON body
            .send()
            .await
            .expect("Patch finish failed");

        assert_eq!(res.status().as_u16(), 200);

        // Deserialize the updated assignment and check if the finished_at was updated
        let updated: JobAssignment = res.json().await.expect("Deserialization failed");
        assert_eq!(
            updated
                .finished_at
                .unwrap()
                .format("%Y-%m-%dT%H:%M:%S")
                .to_string(),
            finish_ts.format("%Y-%m-%dT%H:%M:%S").to_string()
        );

        // Cleanup
        delete_job_via_api(&client, job.id).await;
        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }
}
