use anyhow::bail;
use reqwest::{Client, StatusCode};
use serde_json::json;

use crate::client::{self, Session};
use crate::models::{Job, JobResult, UserResponse};

/// Register a new user via `/users` rocket endpoint
/// # Arguments
/// * client: An authenticated client, can be retrieved with `client::build_authed_client()`
/// * base_url: The base url of the swarm server, excluding the port number e.g. http://127.0.0.1
/// * username: The desired username
/// * email: The desired email
/// * password: The desired password, (hashed and salted during procedure)
/// # Returns
/// result: A UserResponse, which contains the created user
/// # Panics
/// doesn't panic but returns error if
/// * username or email already exists
/// * password doesn't meet requirements
/// * migrations not ran and/or server not running
/// # Examples
/// Examples
/// ```
/// let client = client::build_authed_client("token").unwrap();
/// let user = commands::register_user(&client, "http://127.0.0.1", "username", "email", "password").await.unwrap();
/// ```
pub async fn register_user(
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

    // If the response had unsuccessful status code, bail with correct message
    if !res.status().is_success() {
        // we interpret the status code and message
        let status = res.status();
        let body_text = res.text().await.unwrap_or_default();

        // The reason for the bad request is that the password doesn't meet requirements
        if status == StatusCode::BAD_REQUEST && body_text.contains("Password must") {
            bail!(
                "Registration failed: {}",
                body_text.trim_matches(|c: char| c == '"' || c.is_whitespace())
            );
        }

        // The reason for the conflict is that the username or email already exists
        if status == StatusCode::CONFLICT {
            bail!("Registration failed: username or email already exists.");
        }

        // Otherwise, something else went wrong
        bail!("Registration failed (status={}): {}", status, body_text);
    }

    // If the response was successful, return the user
    Ok(res.json().await?)
}

/// Login via `/login` and then fetch the user via `/users/username/<name>`
/// # Arguments
/// * client: An authenticated client, can be retrieved with `client::build_authed_client()`
/// * base_url: The base url of the swarm server, excluding the port number e.g. http://127.0.0.1
/// * username: The desired username
/// * password: The desired password (hashed and salted during procedure)
/// # Returns
/// result: A tuple of the token and UserResponse
/// # Panics
/// doesn't panic but returns error if
/// * username or password invalid
/// * migrations not ran and/or server not running
/// # Examples
/// ```
/// let client = client::build_authed_client("token").unwrap();
/// let (token, user) = commands::login_user(&client, "http://127.0.0.1", "username", "password").await.unwrap();
/// ```
pub async fn login_user(
    client: &Client,
    base_url: &str,
    username: &str,
    password: &str,
) -> anyhow::Result<(String, UserResponse)> {
    // Post to /login
    let res = client
        .post(&format!("{}/login", base_url))
        .json(&json!({ "username": username, "password": password }))
        .send()
        .await?;

    // If authgaurd failed meaning the username or password was incorrect
    if res.status() == StatusCode::UNAUTHORIZED {
        bail!("Login failed: invalid username or password.");
    }

    // If the response had unsuccessful status code, bail with correct message
    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        bail!("Login request failed (status={}): {}", status, body);
    }

    // Extract the token
    let resp_json: serde_json::Value = res.json().await?;
    let token = resp_json["token"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No token in response"))?
        .to_string();

    // Build authed client
    let authed_client = client::build_authed_client(&token)?;

    // Fetch the user
    let user_resp = authed_client
        .get(format!("{}/users/username/{}", base_url, username))
        .send()
        .await?;

    // If the response had unsuccessful status code, bail with correct message
    if !user_resp.status().is_success() {
        bail!("Failed to fetch user info (status={})", user_resp.status());
    }

    // endpoint returns Option<UserResponse>
    let user_opt: Option<UserResponse> = user_resp.json().await?;
    let user = user_opt.ok_or_else(|| anyhow::anyhow!("User lookup returned None"))?;

    Ok((token, user))
}

/// PUT /users/<id>  ➜ update username / email / password
/// # Arguments
/// * session: An authenticated session, can be retrieved with `client::build_authed_client()`
/// * new_username: The desired username
/// * new_email: The desired email
/// * new_password: The desired password (hashed and salted during procedure)
/// # Returns
/// result: A UserResponse
/// # Panics
/// doesn't panic but returns error if
/// * username or password invalid
/// * migrations not ran and/or server not running
/// # Examples
/// ```
/// let client = client::build_authed_client("token").unwrap();
/// let user = commands::update_user(&client, "http://127.0.0.1", "username", "email", "password").await.unwrap();
/// ```
pub async fn update_user(
    session: &Session,
    new_username: &str,
    new_email: &str,
    new_password: Option<&str>,
) -> anyhow::Result<UserResponse> {
    // Construct payload
    let mut payload = json!({
        "username": new_username,
        "email": new_email,
    });

    // Add password if provided
    if let Some(pw) = new_password {
        payload["password"] = json!(pw);
    }

    // Send update http request
    let res = session
        .client
        .put(format!("{}/users/{}", session.app_host, session.user.id))
        .json(&payload)
        .send()
        .await?;

    // If the response had unsuccessful status code, bail with correct message
    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        anyhow::bail!("Update failed (status={}): {}", status, body);
    }

    Ok(res.json().await?)
}

/// Create a new job via `POST /jobs`
/// # Arguments
/// * session: An authenticated session, can be retrieved with `client::build_authed_client()`
/// * job_name: The name of the job
/// * image_url: The URL of the image
/// * image_format: The format of the image
/// * output_type: The type of output
/// * output_paths: The paths of the output
/// * schedule_type: The type of schedule
/// * cron_expression: The cron expression | ie. "0 5 * * *"
/// # Returns
/// result: A JobResponse
/// # Panics
/// doesn't panic but returns error if
/// * session not authenticated or malformed
/// * migrations not ran and/or server not running
/// # Examples
/// ```
/// let client = client::build_authed_client("token").unwrap();
/// let job = commands::submit_job(&client, "http://127.0.0.1", "job_name", "image_url", "image_format", "output_type", "output_paths", "schedule_type", "cron_expression").await.unwrap();
/// ```
pub async fn submit_job(
    session: &Session,
    job_name: &str,
    image_url: &str,
    image_format: &str,
    output_type: &str,
    output_paths: Option<Vec<String>>,
    schedule_type: &str,
    cron_expression: Option<&str>,
) -> anyhow::Result<Job> {
    // Determine initial state
    let initial_state = if schedule_type == "Cron" {
        "Submitted"
    } else {
        "Queued"
    };

    let payload = json!({
        "user_id": session.user.id,
        "job_name": job_name,
        "image_url": image_url,
        "image_format": image_format,
        "docker_flags": null,
        "output_type": output_type,
        "output_paths": output_paths,
        "schedule_type": schedule_type,
        "cron_expression": cron_expression,
        "notes": null,
        "state": initial_state
    });

    // Send POST request
    let res = session
        .client
        .post(format!("{}/jobs", session.app_host))
        .json(&payload)
        .send()
        .await?;

    // If the response had unsuccessful status code, bail with correct message
    if res.status() != StatusCode::CREATED {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        bail!("Job submission failed (status={}): {}", status, body);
    }

    Ok(res.json().await?)
}

/// List all jobs for the current user
/// # Arguments
/// * session: An authenticated session, can be retrieved with `client::build_authed_client()`
/// # Returns
/// result: A list of jobs
/// # Panics
/// doesn't panic but returns error if
/// * session not authenticated or malformed
/// * migrations not ran and/or server not running
/// # Examples
/// ```
/// let client = client::build_authed_client("token").unwrap();
/// let jobs = commands::list_jobs(&client).await.unwrap();
/// ```
pub async fn list_jobs(session: &Session) -> anyhow::Result<Vec<Job>> {
    let url = format!(
        "{}/jobs/by_admin?user_id={}",
        session.app_host, session.user.id
    );
    let res = session.client.get(url).send().await?;
    if !res.status().is_success() {
        bail!("Failed to list jobs (status={})", res.status());
    }
    Ok(res.json().await?)
}

/// Filter finished jobs (not Submitted / Queued / Running)
/// # Arguments
/// * session: An authenticated session, can be retrieved with `client::build_authed_client()`
/// # Returns
/// result: A list of finished jobs
/// # Panics
/// doesn't panic but returns error if
/// * session not authenticated or malformed
/// * migrations not ran and/or server not running
/// # Examples
/// ```
/// let client = client::build_authed_client("token").unwrap();
/// let jobs = commands::get_finished_jobs(&client).await.unwrap();
/// ```
pub async fn get_finished_jobs(session: &Session) -> anyhow::Result<Vec<Job>> {
    let jobs = list_jobs(session).await?;
    Ok(jobs
        .into_iter()
        .filter(|j| {
            let s = j.state.as_str();
            s != "Submitted" && s != "Queued" && s != "Running"
        })
        .collect())
}

/// Fetch `/results/job/<id>`
/// # Arguments
/// * session: An authenticated session, can be retrieved with `client::build_authed_client()`
/// * job_id: The id of the job
/// # Returns
/// result: A list of job results
/// # Panics
/// doesn't panic but returns error if
/// * session not authenticated or malformed
/// * migrations not ran and/or server not running
/// * job_id not found
/// # Examples
/// ```
/// let client = client::build_authed_client("token").unwrap();
/// let results = commands::get_results_for_job(&client, 1).await.unwrap();
/// ```
pub async fn get_results_for_job(session: &Session, job_id: i32) -> anyhow::Result<Vec<JobResult>> {
    let url = format!("{}/results/job/{}", session.app_host, job_id);
    let res = session.client.get(url).send().await?;

    // If not found, no results where fetched -> job not finished yet
    if res.status() == StatusCode::NOT_FOUND {
        return Ok(vec![]); // none yet
    }

    // If the response had unsuccessful status code, bail with correct message
    if !res.status().is_success() {
        bail!(
            "Failed to fetch results for job {} (status={})",
            job_id,
            res.status()
        );
    }
    Ok(res.json().await?)
}
