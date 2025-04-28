/* ===================== ⚙️ User API Overview =====================

== 🛠️ CRUD ==
• GET     /users/:id                         → Fetch user by ID                       → 200 OK (User)
• POST    /users                             → Create new user (NewUser)             → 201 Created (User)
• DELETE  /users/:id                         → Delete user by ID                     → 204 No Content
• PUT     /users/:id                         → Update user by ID                     → 200 OK (User)

== 🔍 Lookup ==
• GET     /users/email/:email                → Find user by email                    → 200 OK (Option<User>)
• GET     /users/username/:username          → Find user by username                 → 200 OK (Option<User>)

== 🔍 Search ==
• GET     /users/search/username?q=:q        → Search users by username              → 200 OK (Vec<User>)
• GET     /users/search/email?q=:q           → Search users by email                 → 200 OK (Vec<User>)

== 📑 Listing ==
• GET     /users?page=:page&limit=:limit     → List all users (paginated)            → 200 OK (Vec<User>)

== ⚡ Existence Checks ==
• HEAD    /users/exists/email/:email         → Exists by email                       → 200 OK / 404 Not Found
• HEAD    /users/exists/username/:username   → Exists by username                    → 200 OK / 404 Not Found

== 🔗 Relational & Aggregation ==
• GET     /users/with-jobs                   → Users with jobs                       → 200 OK (Vec<User>)
• GET     /users/job-counts                  → User job counts                       → 200 OK (Vec<(User, i64)>)

======================================================================== */

use reqwest::StatusCode;
use rocket::serde::json::json;
use swarmsync_core::database::models::user::UserResponse;
use tokio;

pub mod common;

#[cfg(test)]
mod user_api_tests {
    use crate::common::APP_HOST;

    use super::*;

    #[tokio::test]
    async fn test_create_user() {
        let (client, admin_user) = common::build_client_with_logged_in_admin().await;

        let resp = client
            .get(format!("{}/users/{}", APP_HOST, admin_user.id))
            .send()
            .await
            .expect("Request failed");
        assert_eq!(resp.status(), StatusCode::OK);

        // Clean up by deleting the user
        common::delete_user_via_api(&client, admin_user.id).await;
    }

    #[tokio::test]
    async fn test_get_user_by_id() {
        // Use the helper function to create an authenticated user and get the token
        let (client, admin_user) = common::build_client_with_logged_in_admin().await;

        // Make an authenticated GET request to fetch the user by ID
        let resp = client
            .get(&format!("{}/users/{}", common::APP_HOST, admin_user.id))
            .send()
            .await
            .unwrap();

        // Assert that the status code is OK
        assert_eq!(resp.status(), StatusCode::OK);

        // Deserialize the response body into the `User` struct
        let fetched: UserResponse = resp.json().await.unwrap();

        // Assert that the fetched user matches the admin user that was created
        assert_eq!(fetched.id, admin_user.id);
        assert_eq!(fetched.username, admin_user.username);
        assert_eq!(fetched.email, admin_user.email);

        // Clean up by deleting the user
        common::delete_user_via_api(&client, admin_user.id).await;
    }

    use serde::Serialize;

    #[derive(Serialize)]
    struct UpdateUserRequest {
        username: String,
        email: String,
        password: Option<String>,
    }

    #[tokio::test]
    async fn test_update_user() {
        let (client, user_resp) = common::build_client_with_logged_in_admin().await;

        // Define new values
        let new_password = "MySecurePass123";
        let new_email = format!("updated_{}@example.com", user_resp.id);

        // Build JSON payload directly
        let update_payload = json!({
            "username": user_resp.username,
            "email": new_email,
            "password": new_password
        });

        // Send PUT request
        let resp = client
            .put(format!("{}/users/{}", APP_HOST, user_resp.id))
            .json(&update_payload)
            .send()
            .await
            .expect("Request failed");

        assert_eq!(resp.status(), StatusCode::OK);

        let updated: UserResponse = resp.json().await.unwrap();
        assert_eq!(updated.id, user_resp.id);
        assert_eq!(updated.email, new_email);

        // Attempt login with new password
        let login_resp = common::login_user(&client, &user_resp.username, new_password).await;
        assert_eq!(
            login_resp.status(),
            StatusCode::OK,
            "Login with new password failed"
        );

        // Attempt login with old password
        let old_pass_login_resp =
            common::login_user(&client, &user_resp.username, common::TEST_PASSWORD).await;
        assert_eq!(
            old_pass_login_resp.status(),
            StatusCode::UNAUTHORIZED,
            "Old password still works"
        );

        common::delete_user_via_api(&client, user_resp.id).await;
    }

    #[tokio::test]
    async fn test_delete_user() {
        let (client, user) = common::build_client_with_logged_in_admin().await;
        let _ = common::delete_user_via_api(&client, user.id).await;
        // delete_user_via_api should return status or panic on error
        // verify deletion by attempting to fetch
        let (client1, user1) = common::build_client_with_logged_in_admin().await;
        let check = client1
            .get(format!("{}/users/{}", APP_HOST, user.id))
            .send()
            .await
            .expect("Request failed");

        assert_eq!(check.status(), StatusCode::NOT_FOUND);
        common::delete_user_via_api(&client1, user1.id).await;
    }

    #[tokio::test]
    async fn test_find_user_by_email() {
        let (client, user) = common::build_client_with_logged_in_admin().await;
        let resp = client
            .get(format!("{}/users/email/{}", APP_HOST, user.email))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let found: Option<UserResponse> = resp.json().await.unwrap();
        assert!(found.is_some());
        let u = found.unwrap();
        assert_eq!(u.id, user.id);
        common::delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_find_user_by_username() {
        let (client, user) = common::build_client_with_logged_in_admin().await;
        let resp = client
            .get(format!("{}/users/username/{}", APP_HOST, user.username))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let found: Option<UserResponse> = resp.json().await.unwrap();
        assert!(found.is_some());
        let u = found.unwrap();
        assert_eq!(u.username, user.username);
        common::delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_search_users_by_username() {
        let (_client, user1) = common::build_client_with_logged_in_admin().await;
        // create a second user for search
        let (client, user2) = common::build_client_with_logged_in_admin().await;

        let query = &user1.username[..3];
        let resp = client
            .get(format!("{}/users/search/username?q={}", APP_HOST, query))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let list: Vec<UserResponse> = resp.json().await.unwrap();
        assert!(list.iter().any(|u| u.id == user1.id));
        common::delete_user_via_api(&client, user1.id).await;
        common::delete_user_via_api(&client, user2.id).await;
    }

    #[tokio::test]
    async fn test_search_users_by_email() {
        let (_client, user1) = common::build_client_with_logged_in_admin().await;
        let (client, user2) = common::build_client_with_logged_in_admin().await;
        let query = "example.com";
        let resp = client
            .get(format!("{}/users/search/email?q={}", APP_HOST, query))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let list: Vec<UserResponse> = resp.json().await.unwrap();
        assert!(list.len() >= 2);
        common::delete_user_via_api(&client, user1.id).await;
        common::delete_user_via_api(&client, user2.id).await;
    }

    #[tokio::test]
    async fn test_list_users_paginated() {
        let (_client, u1) = common::build_client_with_logged_in_admin().await;
        let (_client, u2) = common::build_client_with_logged_in_admin().await;
        let (client, u3) = common::build_client_with_logged_in_admin().await;

        let resp = client
            .get(format!("{}/users?page=1&limit=2", APP_HOST))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let page1: Vec<UserResponse> = resp.json().await.unwrap();
        assert_eq!(page1.len(), 2);

        let resp2 = client
            .get(format!("{}/users?page=2&limit=2", APP_HOST))
            .send()
            .await
            .unwrap();

        let page2: Vec<UserResponse> = resp2.json().await.unwrap();
        println!("Page 2 users: {:?}", page2);
        assert!(page2.len() > 0); // Check that page2 has users

        common::delete_user_via_api(&client, u1.id).await;
        common::delete_user_via_api(&client, u2.id).await;
        common::delete_user_via_api(&client, u3.id).await;
    }

    #[tokio::test]
    async fn test_exists_user_by_email() {
        let (client, user) = common::build_client_with_logged_in_admin().await;
        let resp = client
            .head(format!("{}/users/exists/email/{}", APP_HOST, user.email))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        let resp2 = client
            .head(format!(
                "{}/users/exists/email/nonexistent@example.com",
                APP_HOST
            ))
            .send()
            .await
            .unwrap();
        assert_eq!(resp2.status(), StatusCode::NOT_FOUND);

        common::delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_exists_user_by_username() {
        let (client, user) = common::build_client_with_logged_in_admin().await;
        let resp = client
            .head(format!(
                "{}/users/exists/username/{}",
                APP_HOST, user.username
            ))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        let resp2 = client
            .head(format!(
                "{}/users/exists/username/nonexistent_user",
                APP_HOST
            ))
            .send()
            .await
            .unwrap();
        assert_eq!(resp2.status(), StatusCode::NOT_FOUND);

        common::delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_users_with_jobs() {
        // Create client, user, and jobs
        let (client, user, _jobs, job_ids) = common::build_client_and_user_with_n_jobs(3).await;

        // Call /users/with-jobs endpoint
        let resp = client
            .get(format!("{}/users/with-jobs", APP_HOST))
            .send()
            .await
            .expect("Request to /users/with-jobs failed");

        assert_eq!(resp.status(), StatusCode::OK);

        let users: Vec<UserResponse> = resp.json().await.expect("Failed to parse user list");
        let found = users.iter().any(|u| u.id == user.id);

        assert!(
            found,
            "Expected user with ID {} to be present in /users/with-jobs response",
            user.id
        );

        // Cleanup: delete jobs, then user
        common::delete_jobs_via_api(&client, &job_ids).await;
        common::delete_user_via_api(&client, user.id).await;
    }

    #[tokio::test]
    async fn test_user_job_counts() {
        // Create 3 users with 1, 2, and 3 jobs respectively
        let (client1, user1, _, jobs1) = common::build_client_and_user_with_n_jobs(1).await;
        let (client2, user2, _, jobs2) = common::build_client_and_user_with_n_jobs(2).await;
        let (client3, user3, _, jobs3) = common::build_client_and_user_with_n_jobs(3).await;

        // Use any client to call the aggregation endpoint
        let resp = client1
            .get(format!("{}/users/job-counts", APP_HOST))
            .send()
            .await
            .expect("Request to /users/job-counts failed");

        assert_eq!(resp.status(), StatusCode::OK);

        let result: Vec<(serde_json::Value, i64)> = resp
            .json()
            .await
            .expect("Failed to parse job count response");

        // Helper to extract user + count
        let get_count_for_id = |id: i32| -> Option<i64> {
            result
                .iter()
                .find(|(user, _count)| user["id"] == id)
                .map(|(_user, count)| *count)
        };

        // Verify counts
        assert_eq!(
            get_count_for_id(user1.id),
            Some(1),
            "Expected 1 job for user1"
        );
        assert_eq!(
            get_count_for_id(user2.id),
            Some(2),
            "Expected 2 jobs for user2"
        );
        assert_eq!(
            get_count_for_id(user3.id),
            Some(3),
            "Expected 3 jobs for user3"
        );

        // Cleanup jobs first
        common::delete_jobs_via_api(&client1, &jobs1).await;
        common::delete_jobs_via_api(&client2, &jobs2).await;
        common::delete_jobs_via_api(&client3, &jobs3).await;

        // Then users
        common::delete_user_via_api(&client1, user1.id).await;
        common::delete_user_via_api(&client2, user2.id).await;
        common::delete_user_via_api(&client3, user3.id).await;
    }
}
