use reqwest;

use crate::commands::http_with_rocket_port;
use crate::model::UserResponse;
use reqwest::{header, Client};

#[derive(Clone)]
pub struct Session {
    pub client: Client,
    pub user: UserResponse,
    pub app_host: String,
}

/// Builds authenticated client from token, token fetched by logging in
pub fn build_authed_client(token: &str) -> anyhow::Result<Client> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&format!("Bearer {}", token))?,
    );

    Ok(Client::builder().default_headers(headers).build()?)
}

/// Simple GET / to ensure the server is up.
pub async fn is_reachable(url: &str) -> anyhow::Result<bool> {
    let http_base = http_with_rocket_port(url)?;
    let resp = reqwest::get(format!("{}/", http_base)).await?;
    Ok(resp.status().is_success())
}
