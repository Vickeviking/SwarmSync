use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::Error;
use argon2::password_hash::SaltString;
use argon2::Argon2;
use argon2::{PasswordHash, PasswordHasher, PasswordVerifier};

use rand::distributions::Alphanumeric;
use rand::Rng;

use crate::database::models::user::User;

/// Struct to hold username and password
#[derive(serde::Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

/// Authorize a user and return a session id
/// # Arguments
/// * `user` - The user to authorize
/// * `credentials` - The credentials to authorize
/// # Returns
/// * `Result<String, Error>` - The session id
/// # Example
/// ```
/// use swarm_core::auth::authorize_user;
///
/// let user = User {
///     username: "username".to_string(),
///     password_hash: "hash".to_string(),
/// };
///
/// let credentials = Credentials {
///     username: "username".to_string(),
///     password: "password".to_string(),
/// };
///
/// let session_id = authorize_user(&user, credentials).unwrap();
/// ```
/// # Panics
/// no panic , but errors if
/// * If the user does not exist
/// * If the password is incorrect

pub fn authorize_user(user: &User, credentials: Credentials) -> Result<String, Error> {
    let argon2 = Argon2::default();
    let db_hash = PasswordHash::new(&user.password_hash)?;
    argon2.verify_password(credentials.password.as_bytes(), &db_hash)?;

    let session_id = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(128)
        .map(char::from)
        .collect();

    Ok(session_id)
}

/// Hash a password with argon2
pub fn hash_password(password: String) -> Result<String, Error> {
    let salt = SaltString::generate(OsRng);
    let argon2 = Argon2::default();
    let hashed_password = argon2.hash_password(password.as_bytes(), &salt)?;

    Ok(hashed_password.to_string())
}

/// Check if a password is valid,
/// checks against password rules
pub fn is_password_valid(password: &str) -> bool {
    let min_len = 8;
    let max_len = 128;

    let has_min_len = password.len() >= min_len;
    let has_max_len = password.len() <= max_len;
    let has_letter = password.chars().any(|c| c.is_alphabetic());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());

    has_min_len && has_max_len && has_letter && has_digit
}
