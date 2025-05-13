use chrono::Utc;
use rocket::serde::json::json;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

use common::database::models::worker::Worker;

pub mod common_test;
#[cfg(test)]
mod worker_api_tests {

    use crate::common_test::{
        build_client_with_logged_in_admin, create_worker_via_api, delete_user_via_api,
        delete_worker_via_api, APP_HOST,
    };

    #[tokio::test]
    async fn test_create_worker() {
        let (client, user) = build_client_with_logged_in_admin().await;
        let worker = create_worker_via_api(&client, user.id).await;

        assert_eq!(worker.user_id, user.id);
        assert_eq!(worker.ip_address, "127.0.0.1");
        assert_eq!(worker.hostname, "test-host");

        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_get_worker_by_id() {
        let (client, user) = build_client_with_logged_in_admin().await;
        let worker = create_worker_via_api(&client, user.id).await;

        let res = client
            .get(&format!("{}/workers/{}", APP_HOST, worker.id))
            .send()
            .await
            .expect("GET /workers/:id failed");
        assert!(res.status().is_success());

        let fetched = res
            .json::<Worker>()
            .await
            .expect("Failed to deserialize worker");
        assert_eq!(fetched.id, worker.id);

        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_delete_worker_by_id() {
        let (client, user) = build_client_with_logged_in_admin().await;
        let worker = create_worker_via_api(&client, user.id).await;

        delete_worker_via_api(&client, worker.id).await;

        let res = client
            .get(&format!("{}/workers/{}", APP_HOST, worker.id))
            .send()
            .await
            .expect("GET after DELETE failed");
        assert_eq!(res.status().as_u16(), 404);

        delete_user_via_api(&client, user.id).await;
    }

    // 🔍 Lookup & Search Endpoints

    #[tokio::test]
    async fn test_get_workers_by_admin_id() {
        let (client, user) = build_client_with_logged_in_admin().await;
        let _ = create_worker_via_api(&client, user.id).await;

        let res = client
            .get(&format!("{}/workers/admin/{}", APP_HOST, user.id))
            .send()
            .await
            .expect("GET /workers/admin/:admin_id failed");

        assert!(res.status().is_success());
        let workers: Vec<Worker> = res.json().await.expect("Deserialize Vec<Worker>");
        assert!(!workers.is_empty());

        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_find_worker_by_label() {
        let (client, user) = build_client_with_logged_in_admin().await;
        let worker = create_worker_via_api(&client, user.id).await;

        let res = client
            .get(&format!("{}/workers/label/{}", APP_HOST, worker.label))
            .send()
            .await
            .expect("GET /workers/label/:label failed");

        assert!(res.status().is_success());
        let found = res
            .json::<Option<Worker>>()
            .await
            .expect("Failed to deserialize Option<Worker>");
        assert_eq!(found.unwrap().id, worker.id);

        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_find_worker_by_ip_address() {
        let (client, user) = build_client_with_logged_in_admin().await;
        let label = format!("worker-{}", Uuid::new_v4());
        let ip = format!("ipv4TEST:{}", Uuid::new_v4());
        let res = client
            .post(&format!("{}/workers", APP_HOST))
            .json(&json!({
                "user_id": user.id,
                "label": label,
                "ip_address": ip.clone(),
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

        let worker = res
            .json::<Worker>()
            .await
            .expect("Failed to deserialize Worker from create response");

        let res = client
            .get(&format!("{}/workers/ip/{}", APP_HOST, worker.ip_address))
            .send()
            .await
            .expect("GET /workers/ip/:ip_address failed");

        assert!(res.status().is_success());
        let found = res
            .json::<Option<Worker>>()
            .await
            .expect("Failed to deserialize Option<Worker>");
        assert_eq!(found.unwrap().id, worker.id);

        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_list_workers_by_admin_paginated() {
        let (client, user) = build_client_with_logged_in_admin().await;
        for _ in 0..3 {
            let _ = create_worker_via_api(&client, user.id).await;
        }

        let res = client
            .get(&format!(
                "{}/workers/admin/{}/list?page=1&per_page=2",
                APP_HOST, user.id
            ))
            .send()
            .await
            .expect("GET /workers/admin/:admin_id/list failed");

        assert!(res.status().is_success());
        let workers: Vec<Worker> = res.json().await.expect("Deserialize Vec<Worker>");
        assert!(workers.len() <= 2);

        delete_user_via_api(&client, user.id).await;
    }

    // 🔄 State Update Endpoints

    #[tokio::test]
    async fn test_update_last_seen_timestamp() {
        let (client, user) = build_client_with_logged_in_admin().await;
        let worker = create_worker_via_api(&client, user.id).await;

        let last_seen_ts = Utc::now().naive_utc();

        // Send the timestamp in a JSON object with the field "last_seen_at"
        let payload = json!({ "last_seen_at": last_seen_ts.to_string() });

        let res = client
            .put(&format!("{}/workers/{}/last-seen", APP_HOST, worker.id))
            .json(&payload) // Send JSON body
            .send()
            .await
            .expect("PUT /workers/:id/last-seen failed");

        assert!(
            res.status().is_success(),
            "Expected 2xx, got: {}",
            res.status()
        );

        let updated = res
            .json::<Worker>()
            .await
            .expect("Failed to deserialize updated worker");

        assert!(
            updated.last_seen_at.is_some(),
            "Expected last_seen_at to be Some"
        );

        assert_eq!(
            updated
                .last_seen_at
                .unwrap()
                .format("%Y-%m-%dT%H:%M:%S")
                .to_string(),
            last_seen_ts.format("%Y-%m-%dT%H:%M:%S").to_string()
        );

        // Cleanup
        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_update_worker() {
        let (client, user) = build_client_with_logged_in_admin().await;
        let worker = create_worker_via_api(&client, user.id).await;

        // Modify the worker's properties
        let updated_worker = Worker {
            id: worker.id,
            user_id: worker.user_id,
            label: "Updated Worker Name".to_string(),
            last_seen_at: worker.last_seen_at, // Keep other fields the same
            // Add other fields you want to test updating
            ip_address: worker.ip_address.clone(),
            hostname: worker.hostname.clone(),
            ssh_user: worker.ssh_user.clone(),
            ssh_key: worker.ssh_key.clone(),
            docker_version: worker.docker_version.clone(),
            arch: worker.arch.clone(),
            os: worker.os.clone(),
            tags: worker.tags.clone(),
            created_at: worker.created_at.clone(),
        };

        let res = client
            .patch(&format!("{}/workers/{}", APP_HOST, worker.id))
            .json(&json!({
                "id": updated_worker.id,
                "user_id": updated_worker.user_id,
                "label": updated_worker.label,
                "last_seen_at": updated_worker.last_seen_at,
                "ip_address": updated_worker.ip_address,
                "hostname": updated_worker.hostname,
                "ssh_user": updated_worker.ssh_user,
                "ssh_key": updated_worker.ssh_key,
                "docker_version": updated_worker.docker_version,
                "arch": updated_worker.arch,
                "os": updated_worker.os,
                "tags": updated_worker.tags,
                "created_at" : updated_worker.created_at,
            }))
            .send()
            .await
            .expect("PATCH /workers/:id failed");

        assert!(
            res.status().is_success(),
            "Expected 2xx, got: {}",
            res.status()
        );

        let updated: Worker = res
            .json()
            .await
            .expect("Failed to deserialize updated worker");

        // Verify the worker data was updated correctly
        assert_eq!(updated.label, "Updated Worker Name");

        // Ensure other fields remain unchanged
        assert_eq!(updated.last_seen_at, worker.last_seen_at);

        // Cleanup
        delete_worker_via_api(&client, worker.id).await;
        delete_user_via_api(&client, user.id).await;
    }
}
