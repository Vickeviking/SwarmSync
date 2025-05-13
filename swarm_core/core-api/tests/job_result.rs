pub mod common_test;

#[cfg(test)]
mod job_result_api_tests {
    use crate::common_test::{
        assign_result_to_job, build_client_and_user_with_n_jobs, delete_job_via_api,
        delete_user_via_api, APP_HOST,
    };
    use common::database::models::job::JobResult;
    use rocket::serde::json::json;

    #[tokio::test]
    async fn test_create_job_result() {
        let (client, user, jobs, _) = build_client_and_user_with_n_jobs(1).await;
        let job = &jobs[0];
        let result = assign_result_to_job(&client, job.id).await;
        assert_eq!(result.job_id, job.id);
        delete_job_via_api(&client, job.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_get_job_result_by_id() {
        let (client, user, jobs, _) = build_client_and_user_with_n_jobs(1).await;
        let job = &jobs[0];
        let result = assign_result_to_job(&client, job.id).await;

        let res = client
            .get(&format!("{}/results/{}", APP_HOST, result.id))
            .send()
            .await
            .unwrap();
        assert!(res.status().is_success());

        let fetched = res.json::<JobResult>().await.unwrap();
        assert_eq!(fetched.id, result.id);

        delete_job_via_api(&client, job.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_delete_job_result() {
        let (client, user, jobs, _) = build_client_and_user_with_n_jobs(1).await;
        let job = &jobs[0];
        let result = assign_result_to_job(&client, job.id).await;

        let res = client
            .delete(&format!("{}/results/{}", APP_HOST, result.id))
            .send()
            .await
            .unwrap();
        assert!(res.status().is_success());

        delete_job_via_api(&client, job.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_get_results_by_job_id() {
        let (client, user, jobs, _) = build_client_and_user_with_n_jobs(1).await;
        let job = &jobs[0];
        assign_result_to_job(&client, job.id).await;

        let res = client
            .get(&format!("{}/results/job/{}", APP_HOST, job.id))
            .send()
            .await
            .unwrap();
        assert!(res.status().is_success());

        let results: Vec<JobResult> = res.json().await.unwrap();
        assert!(!results.is_empty());

        delete_job_via_api(&client, job.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_list_results_for_job() {
        let (client, user, jobs, _) = build_client_and_user_with_n_jobs(1).await;
        let job = &jobs[0];
        assign_result_to_job(&client, job.id).await;

        let res = client
            .get(&format!("{}/results/list/{}", APP_HOST, job.id))
            .send()
            .await
            .unwrap();
        assert!(res.status().is_success());

        let list: Vec<JobResult> = res.json().await.unwrap();
        assert!(!list.is_empty());

        delete_job_via_api(&client, job.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_get_most_recent_result_for_job() {
        let (client, user, jobs, _) = build_client_and_user_with_n_jobs(1).await;
        let job = &jobs[0];
        assign_result_to_job(&client, job.id).await;

        let res = client
            .get(&format!("{}/results/recent/{}", APP_HOST, job.id))
            .send()
            .await
            .unwrap();
        assert!(res.status().is_success());

        let result: Option<JobResult> = res.json().await.unwrap();
        assert!(result.is_some());

        delete_job_via_api(&client, job.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_update_stdout_field() {
        let (client, user, jobs, _) = build_client_and_user_with_n_jobs(1).await;
        let job = &jobs[0];
        let result = assign_result_to_job(&client, job.id).await;

        let new_stdout = "Updated output logs";
        let res = client
            .patch(&format!("{}/results/{}/stdout", APP_HOST, result.id))
            .json(&json!({ "stdout": new_stdout }))
            .send()
            .await
            .unwrap();
        assert!(res.status().is_success());

        let updated = res.json::<JobResult>().await.unwrap();
        assert_eq!(updated.stdout.as_deref(), Some(new_stdout));

        delete_job_via_api(&client, job.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_update_files_field() {
        let (client, user, jobs, _) = build_client_and_user_with_n_jobs(1).await;
        let job = &jobs[0];
        let result = assign_result_to_job(&client, job.id).await;

        let new_files = vec![Some("final-output.csv".to_string())];
        let res = client
            .patch(&format!("{}/results/{}/files", APP_HOST, result.id))
            .json(&json!({ "files": new_files }))
            .send()
            .await
            .unwrap();
        assert!(res.status().is_success());

        let updated = res.json::<JobResult>().await.unwrap();
        assert_eq!(updated.files.unwrap(), new_files);

        delete_job_via_api(&client, job.id).await;
        delete_user_via_api(&client, user.id).await;
    }
}
