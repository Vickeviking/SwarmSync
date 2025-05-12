use anyhow::{anyhow, bail, Context, Result};
use reqwest::{Client, StatusCode};
use serde_json::json;

use super::model::{Job, JobResult, UserResponse, WorkerStatusEnum};
use super::net::{self, http_with_rocket_port, Session};

/// Register a new user via `/users`
pub async fn register_user(
    client: &Client,
    base_url: &str,
    username: &str,
    email: &str,
    password: &str,
) -> anyhow::Result<UserResponse> {
    let http_base = http_with_rocket_port(base_url)?;
    let url = format!("{http_base}/users");
    println!("DEBUG register_user → {}", url); // <‑‑👀

    let res = client
        .post(&url)
        .json(&json!({ "username": username, "email": email, "password": password }))
        .send()
        .await
        .with_context(|| format!("POST {url} failed"))?; // <‑‑ context includes URL

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        anyhow::bail!("register_user {url} → {status}: {body}");
    }
    Ok(res.json().await.context("bad JSON in /users response")?)
}

pub async fn login_user(
    client: &Client,
    base_url: &str,
    username: &str,
    password: &str,
) -> anyhow::Result<(String, UserResponse)> {
    let http_base = http_with_rocket_port(base_url)?;
    let login_url = format!("{http_base}/login");
    println!("DEBUG login_user → {}", login_url); // <‑‑👀

    let res = client
        .post(&login_url)
        .json(&json!({ "username": username, "password": password }))
        .send()
        .await
        .with_context(|| format!("POST {login_url} failed"))?;

    if res.status() == StatusCode::UNAUTHORIZED {
        anyhow::bail!("login_user {login_url} → 401 UNAUTHORIZED");
    }
    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        anyhow::bail!("login_user {login_url} → {status}: {body}");
    }

    let token = res
        .json::<serde_json::Value>()
        .await
        .context("malformed JSON in /login response")?["token"]
        .as_str()
        .ok_or_else(|| anyhow!("no token in /login response"))?
        .to_string();

    let authed = net::build_authed_client(&token)?;
    let user_url = format!("{http_base}/users/username/{username}");
    println!("DEBUG fetch_user → {}", user_url); // <‑‑👀

    let res = authed
        .get(&user_url)
        .send()
        .await
        .with_context(|| format!("GET {user_url} failed"))?;

    if !res.status().is_success() {
        anyhow::bail!("fetch_user {user_url} → {}", res.status());
    }

    let user_opt: Option<UserResponse> = res
        .json()
        .await
        .context("bad JSON in /users/username response")?;
    let user = user_opt.ok_or_else(|| anyhow!("user not found"))?;

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

    let http_base = http_with_rocket_port(&session.app_host)?;
    let res = session
        .client
        .put(format!("{}/users/{}", http_base, session.user.id))
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

    let http_base = http_with_rocket_port(&session.app_host)?;
    let res = session
        .client
        .post(format!("{}/jobs", http_base))
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
    let http_base = http_with_rocket_port(&session.app_host)?;
    let url = format!("{}/jobs/by_admin?user_id={}", http_base, session.user.id);
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
    let http_base = http_with_rocket_port(&session.app_host)?;
    let url = format!("{}/results/job/{}", http_base, job_id);
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

/// Fetch workerStatusEnum from worker_id

pub async fn get_worker_status(session: &Session, worker_id: i32) -> Result<WorkerStatusEnum> {
    let http_base = http_with_rocket_port(&session.app_host)?;
    let url = format!("{}/worker-status/worker/{}", http_base, worker_id);

    let resp = session
        .client
        .get(&url)
        .send()
        .await
        .context("failed to send GET to worker-status endpoint")?;

    if resp.status() == StatusCode::NOT_FOUND {
        bail!("worker {} not found (404)", worker_id);
    }

    let resp = resp
        .error_for_status()
        .with_context(|| format!("error status fetching worker {}", worker_id))?;

    let status = resp
        .json::<WorkerStatusEnum>()
        .await
        .context("failed to parse WorkerStatusEnum from JSON")?;

    Ok(status)
}
