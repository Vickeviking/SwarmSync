use crate::client::{build_authed_client, Session};
use crate::models::UserResponse;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use reqwest::Client;
use serde_json::json;

pub async fn auth_flow(base_url: &str) -> anyhow::Result<Session> {
    let options = vec!["Login", "Register"];
    let choice = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Login or Register?")
        .items(&options)
        .interact()?;

    let username: String = Input::new().with_prompt("Username").interact_text()?;
    let password: String = Input::new().with_prompt("Password").interact_text()?;

    let client = Client::new();

    let (token, user) = match choice {
        0 => login_user(&client, base_url, &username, &password).await?,
        1 => {
            let email: String = Input::new().with_prompt("Email").interact_text()?;
            let user = create_user(&client, base_url, &username, &email, &password).await?;
            let (token, _) = login_user(&client, base_url, &username, &password).await?;
            (token, user)
        }
        _ => unreachable!(),
    };

    let client = build_authed_client(&token)?;
    Ok(Session {
        client,
        user,
        app_host: base_url.to_string(),
    })
}

async fn create_user(
    client: &Client,
    base_url: &str,
    username: &str,
    email: &str,
    password: &str,
) -> anyhow::Result<UserResponse> {
    let res = client
        .post(&format!("{}/users", base_url))
        .json(&json!({
            "username": username,
            "email": email,
            "password": password
        }))
        .send()
        .await?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res
            .text()
            .await
            .unwrap_or_else(|_| "<unreadable body>".into());
        anyhow::bail!("User creation failed (status={status}): {body}");
    }

    Ok(res.json().await?)
}

async fn login_user(
    client: &Client,
    base_url: &str,
    username: &str,
    password: &str,
) -> anyhow::Result<(String, UserResponse)> {
    let res = client
        .post(&format!("{}/login", base_url))
        .json(&json!({ "username": username, "password": password }))
        .send()
        .await?;

    let json: serde_json::Value = res.json().await?;
    let token = json["token"].as_str().unwrap().to_string();

    // Basic smoke test
    let client = build_authed_client(&token)?;
    let user: UserResponse = client
        .get(format!("{}/users/{}", base_url, json["user_id"]))
        .send()
        .await?
        .json()
        .await?;

    Ok((token, user))
}
