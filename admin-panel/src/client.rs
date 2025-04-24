use crate::models::UserResponse;
use reqwest::{header, Client};

#[derive(Clone)]
pub struct Session {
    pub client: Client,
    pub user: UserResponse,
    pub app_host: String,
}

pub fn build_authed_client(token: &str) -> anyhow::Result<Client> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&format!("Bearer {}", token))?,
    );

    Ok(Client::builder().default_headers(headers).build()?)
}
