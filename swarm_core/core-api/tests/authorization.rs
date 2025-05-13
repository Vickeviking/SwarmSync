use reqwest::{header, StatusCode};

use common::database::models::user::UserResponse;
use common_test::{APP_HOST, TEST_PASSWORD};

pub mod common_test;

#[tokio::test]
async fn test_login() -> anyhow::Result<()> {
    // 1) Setup: unique username + shared client + create user
    let username = common_test::generate_unique_username();
    let client = common_test::http_client();
    let user: UserResponse = common_test::create_user_via_api(&client, &username).await?;

    // 2) Login: correct password
    let resp = common_test::login_user(&client, &username, TEST_PASSWORD).await?;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = resp.json().await.unwrap();
    let token = body["token"].as_str().unwrap();
    assert_eq!(token.len(), 128);

    // 3) Login: incorrect password
    let resp = common_test::login_user(&client, &username, "wrong_password").await?;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // 4) Access protected route without token → expect failure
    let resp = client
        .get(format!("{}/users/{}", APP_HOST, user.id))
        .send()
        .await
        .expect("Request failed");
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // 5) Setup: Authenticated client with Bearer token
    let header_value = format!("Bearer {}", token);
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&header_value).unwrap(),
    );
    let authorized_client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    // 6) Access protected route with token → expect success
    let resp = authorized_client
        .get(format!("{}/users/{}", APP_HOST, user.id))
        .send()
        .await
        .expect("Request failed");
    assert_eq!(resp.status(), StatusCode::OK);

    let fetched_user: UserResponse = resp.json().await.unwrap();
    assert_eq!(fetched_user.id, user.id);
    assert_eq!(fetched_user.username, username);

    // 7) Cleanup: delete user using authorized client
    common_test::delete_user_via_api(&authorized_client, user.id).await?;
    Ok(())
}
