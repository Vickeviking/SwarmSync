use anyhow::bail;
use reqwest::{Client, StatusCode};
use serde_json::json;

use crate::client::{self, Session};
use crate::models::{Job, JobResult, UserResponse};

/// Register a new user via `/users`
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

    if !res.status().is_success() {
        let status = res.status();
        let body_text = res.text().await.unwrap_or_default();

        if status == StatusCode::BAD_REQUEST && body_text.contains("Password must") {
            bail!(
                "Registration failed: {}",
                body_text.trim_matches(|c: char| c == '"' || c.is_whitespace())
            );
        }
        if status == StatusCode::CONFLICT {
            bail!("Registration failed: username or email already exists.");
        }
        bail!("Registration failed (status={}): {}", status, body_text);
    }

    Ok(res.json().await?)
}

/// Login via `/login` and then fetch the user via `/users/username/<name>`
pub async fn login_user(
    client: &Client,
    base_url: &str,
    username: &str,
    password: &str,
) -> anyhow::Result<(String, UserResponse)> {
    // ── 1. POST /login ────────────────────────────────────────────────
    let res = client
        .post(&format!("{}/login", base_url))
        .json(&json!({ "username": username, "password": password }))
        .send()
        .await?;

    if res.status() == StatusCode::UNAUTHORIZED {
        bail!("Login failed: invalid username or password.");
    }
    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        bail!("Login request failed (status={}): {}", status, body);
    }

    let resp_json: serde_json::Value = res.json().await?;
    let token = resp_json["token"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No token in response"))?
        .to_string();

    // ── 2. Build authed client & GET /users/username/<username> ───────
    let authed_client = client::build_authed_client(&token)?;

    let user_resp = authed_client
        .get(format!("{}/users/username/{}", base_url, username))
        .send()
        .await?;

    if !user_resp.status().is_success() {
        bail!("Failed to fetch user info (status={})", user_resp.status());
    }

    // endpoint returns Option<UserResponse>
    let user_opt: Option<UserResponse> = user_resp.json().await?;
    let user = user_opt.ok_or_else(|| anyhow::anyhow!("User lookup returned None"))?;

    Ok((token, user))
}

/// PUT /users/<id>  ➜ update username / email / password
pub async fn update_user(
    session: &Session,
    new_username: &str,
    new_email: &str,
    new_password: Option<&str>,
) -> anyhow::Result<UserResponse> {
    let mut payload = json!({
        "username": new_username,
        "email": new_email,
    });
    if let Some(pw) = new_password {
        payload["password"] = json!(pw);
    }

    let res = session
        .client
        .put(format!("{}/users/{}", session.app_host, session.user.id))
        .json(&payload)
        .send()
        .await?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        anyhow::bail!("Update failed (status={}): {}", status, body);
    }

    Ok(res.json().await?)
}

/// Submit a new job via `POST /jobs`
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

    let res = session
        .client
        .post(format!("{}/jobs", session.app_host))
        .json(&payload)
        .send()
        .await?;

    if res.status() != StatusCode::CREATED {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        bail!("Job submission failed (status={}): {}", status, body);
    }

    Ok(res.json().await?)
}

/// List all jobs for the current user
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
pub async fn get_results_for_job(session: &Session, job_id: i32) -> anyhow::Result<Vec<JobResult>> {
    let url = format!("{}/results/job/{}", session.app_host, job_id);
    let res = session.client.get(url).send().await?;
    if res.status() == StatusCode::NOT_FOUND {
        return Ok(vec![]); // none yet
    }
    if !res.status().is_success() {
        bail!(
            "Failed to fetch results for job {} (status={})",
            job_id,
            res.status()
        );
    }
    Ok(res.json().await?)
}
