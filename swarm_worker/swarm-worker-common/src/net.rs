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

/// Turn “core” or “http://core” or “http://core:8080”
/// into “http://core:8000”
pub fn http_with_rocket_port(base: &str) -> Result<String> {
    let mut url = Url::parse(base)
        .or_else(|_| Url::parse(&format!("http://{base}"))) // bare “core”
        .with_context(|| format!("BASE_URL “{base}” is not a valid URL"))?;

    // Only add port if none already present
    if url.port().is_none() {
        url.set_port(Some(8000))
            .map_err(|_| anyhow!("could not set :8000 on {url}"))?;
    }
    Ok(url.to_string().trim_end_matches('/').to_string())
}
