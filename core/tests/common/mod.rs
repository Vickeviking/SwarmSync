use diesel_async::AsyncPgConnection;
use reqwest::{header, Client, ClientBuilder, StatusCode};
use serde_json::json;
use swarmsync_core::commands;
use swarmsync_core::database::models::user::{User, UserResponse};
use swarmsync_core::database::repositories::user::UserRepository;
use uuid::Uuid;

pub const APP_HOST: &str = "http://localhost:8000";
pub const TEST_PASSWORD: &str = "ADMINPASSWORD123";

/// Single shared HTTP client
pub fn http_client() -> Client {
    Client::new()
}

/// Generate a unique username
pub fn generate_unique_username() -> String {
    format!("test_admin_{}", Uuid::new_v4())
}

/// Create a user via HTTP API, returning the deserialized User
pub async fn create_user_via_api(client: &Client, username: &str) -> UserResponse {
    let email = format!("{}@example.com", username);
    let res = client
        .post(&format!("{}/users", APP_HOST))
        .json(&json!({ "username": username, "email": email, "password": TEST_PASSWORD }))
        .send()
        .await
        .expect("Failed to send create-user request");

    assert!(
        res.status().is_success(),
        "User creation failed (status={}): {:?}",
        res.status(),
        res.text().await.unwrap_or_default()
    );

    // **Await** the JSON deserialization
    res.json::<UserResponse>()
        .await
        .expect("Failed to deserialize User from create response")
}

pub async fn get_full_user_model(user_id: i32) -> User {
    let mut conn: AsyncPgConnection = commands::load_db_connection().await;
    UserRepository::find_by_id(&mut conn, user_id)
        .await
        .expect("Failed to fetch user by id")
}

/// Attempt login via HTTP API, returning the Response
pub async fn login_user(client: &Client, username: &str, password: &str) -> reqwest::Response {
    client
        .post(&format!("{}/login", APP_HOST))
        .json(&json!({ "username": username, "password": password }))
        .send()
        .await
        .expect("Login request failed")
}

pub async fn build_client_with_logged_in_admin() -> (Client, UserResponse) {
    let client = Client::new();

    // 1) Generate a unique username and create the user
    let username = generate_unique_username();
    let user: UserResponse = create_user_via_api(&client, &username).await;

    // 2) Log in the newly created user
    let login_resp = login_user(&client, &username, TEST_PASSWORD).await;
    assert_eq!(login_resp.status(), StatusCode::OK);

    // 3) Deserialize the token from the login response
    let body: serde_json::Value = login_resp.json().await.unwrap();
    let token = body["token"].as_str().unwrap().to_string();

    let header_value = format!("Bearer {}", token);
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&header_value).unwrap(),
    );
    let authorized_client = ClientBuilder::new()
        .default_headers(headers)
        .build()
        .unwrap();
    // 4) insert token into client
    (authorized_client, user)
}

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

/// ======== Job creation ===============

/// Generates a unique job name using a UUID.
pub fn generate_unique_job_name() -> String {
    format!("test_job_{}", Uuid::new_v4())
}

/// Helper to delete a job via the API by ID.
pub async fn delete_job_via_api(client: &Client, job_id: i32) {
    let resp = client
        .delete(&format!("{}/jobs/{}", APP_HOST, job_id))
        .send()
        .await
        .expect("Failed to delete job");
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

/// Deletes multiple jobs given their IDs.
pub async fn delete_jobs_via_api(client: &Client, job_ids: &[i32]) {
    for &id in job_ids {
        delete_job_via_api(client, id).await;
    }
}

pub async fn delete_users_via_api(client: &Client, user_ids: &[i32]) {
    for &id in user_ids {
        delete_user_via_api(client, id).await;
    }
}
