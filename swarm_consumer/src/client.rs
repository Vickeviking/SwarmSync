use crate::models::UserResponse;
use reqwest::{header, Client};

/// Stores session details, such as
#[derive(Clone)]
pub struct Session {
    // Client: Authenticated client
    pub client: Client,
    // User: The currently logged in user
    pub user: UserResponse,
    // App Host: The address for which the core is being ran, excluding portnumber
    pub app_host: String,
}

/// Builds an authenticated client
/// # Arguments
/// * token: bearer token, used for authenticated, gotten from login()
/// # Returns
/// An authenticated client wrapped in an anyhow Result
/// # Panics
/// Doesn't panic, but returns Error if
/// * token invalid
/// * client builder failed, often caused by invalid token
pub fn build_authed_client(token: &str) -> anyhow::Result<Client> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&format!("Bearer {}", token))?,
    );

    Ok(Client::builder().default_headers(headers).build()?)
}
