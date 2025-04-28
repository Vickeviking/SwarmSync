use std::fmt;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// User information returned by the backend (sanitized, without password).
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: NaiveDateTime,
}

/// Job model returned from the backend.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Job {
    pub id: i32,
    pub user_id: i32,
    pub job_name: String,
    pub image_url: String,
    pub image_format: String,
    pub docker_flags: Option<String>,
    pub output_type: String,
    pub output_paths: Option<Vec<String>>,
    pub schedule_type: String,
    pub cron_expression: Option<String>,
    pub notes: Option<String>,
    pub state: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Job result model returned from the backend.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct JobResult {
    pub id: i32,
    pub job_id: i32,
    pub stdout: Option<String>,
    pub files: Option<Vec<String>>,
    pub created_at: NaiveDateTime,
}

/// Worker status enum
#[derive(Debug, Serialize, Deserialize)]
pub enum WorkerStatusEnum {
    Idle,
    Busy,
    Offline,
    Unreachable,
}
impl fmt::Display for WorkerStatusEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            WorkerStatusEnum::Idle => "Idle",
            WorkerStatusEnum::Busy => "Busy",
            WorkerStatusEnum::Offline => "Offline",
            WorkerStatusEnum::Unreachable => "Unreachable",
        };
        write!(f, "{}", s)
    }
}
