use chrono::NaiveDateTime;
use serde::Deserialize;

/// User information returned by the backend (sanitized, without password).
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct UserResponse {
    // id: primary key
    pub id: i32,
    // username: unique, non-null
    pub username: String,
    // email: unique, non-null
    pub email: String,
    // created_at: non-null
    pub created_at: NaiveDateTime,
}

/// Job model returned from the backend.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Job {
    // id: primary key
    pub id: i32,
    // user_id: foreign key
    pub user_id: i32,
    // job_name: non-null
    pub job_name: String,
    // image_url: non-null
    pub image_url: String,
    // image_format: non-null
    pub image_format: String,
    // docker_flags
    pub docker_flags: Option<String>,
    // output_type: non-null
    pub output_type: String,
    // output_paths
    pub output_paths: Option<Vec<String>>,
    // schedule_type: non-null
    pub schedule_type: String,
    // cron_expression
    pub cron_expression: Option<String>,
    // notes
    pub notes: Option<String>,
    // state : non-null
    pub state: String,
    // created_at: non-null
    pub created_at: NaiveDateTime,
    // updated_at: non-null
    pub updated_at: NaiveDateTime,
}

/// Job result model returned from the backend.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct JobResult {
    // id: primary key
    pub id: i32,
    // job_id: foreign key
    pub job_id: i32,
    pub stdout: Option<String>,
    pub files: Option<Vec<String>>,
    pub created_at: NaiveDateTime,
}
