use std::fs;

use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

use crate::client::{build_authed_client, Session};
use crate::commands;
use crate::models::UserResponse;
use crate::views::connect::{config_file_path, CoreConfig};

/// Prompt, authenticate (login / register), persist last username, return Session.
/// # Args
/// * `base_url` - The base URL of the Core instance, no port, e.g. `http://127.0.0.1`
/// # Returns
/// * `Session` - A session struct with the client, user, and app host
/// # Example
/// ```
/// let session = auth_flow("http://127.0.0.1").await.unwrap();
/// ```
pub async fn auth_flow(base_url: &str) -> anyhow::Result<Session> {
    // Read saved username
    let saved_username = fs::read_to_string(config_file_path())
        .ok()
        .and_then(|s| serde_json::from_str::<CoreConfig>(&s).ok())
        .and_then(|c| c.last_username);

    // Prompt for login or register
    let mode_items = ["Login", "Register"];
    let mode = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Authentication")
        .items(&mode_items)
        .default(0)
        .interact()?;

    // Username prompt
    let username = if let Some(ref u) = saved_username {
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Login as '{}'?", u))
            .default(true)
            .interact()?
        {
            u.clone()
        } else {
            Input::new().with_prompt("Username").interact_text()?
        }
    } else {
        Input::new().with_prompt("Username").interact_text()?
    };

    // Password prompt
    let password: String = Input::new().with_prompt("Password").interact_text()?;

    // Login or register flow
    let client = reqwest::Client::new();
    let (token, user): (String, UserResponse) = if mode == 0 {
        // Login
        commands::login_user(&client, base_url, &username, &password).await?
    } else {
        // Register
        let email: String = Input::new().with_prompt("Email").interact_text()?;
        let user = commands::register_user(&client, base_url, &username, &email, &password).await?;
        let (token, _) = commands::login_user(&client, base_url, &username, &password).await?;
        (token, user)
    };

    // Save last username
    let mut cfg = fs::read_to_string(config_file_path())
        .ok()
        .and_then(|s| serde_json::from_str::<CoreConfig>(&s).ok())
        .unwrap_or_else(|| CoreConfig {
            base_url: base_url.to_string(),
            last_username: None,
        });
    cfg.last_username = Some(user.username.clone());
    fs::write(config_file_path(), serde_json::to_string_pretty(&cfg)?)?;

    // Build authed client, return session
    let authed_client = build_authed_client(&token)?;
    Ok(Session {
        client: authed_client,
        user,
        app_host: base_url.to_string(),
    })
}
