use std::fmt;

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::models::{user::User, worker::Worker};
use crate::database::schema::*;
use crate::enums::{
    image_format::ImageFormatEnum, job::JobStateEnum, output::OutputTypeEnum,
    schedule::ScheduleTypeEnum,
};

// A job bound to a user
// Can be sent as JSON over websocket, and stored in postgres
#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(User))] // FK: user_id
pub struct Job {
    pub id: i32,
    // FK
    pub user_id: i32,
    // Job name
    pub job_name: String,
    // Docker image url, where it is found
    pub image_url: String,
    // Docker image format, either tarball or dockerfile
    pub image_format: ImageFormatEnum,
    // Docker flags, how dockerfile should be ran
    pub docker_flags: Option<Vec<Option<String>>>,
    // Output type, either stdout or files
    pub output_type: OutputTypeEnum,
    // If files, list files from where to collect
    pub output_paths: Option<Vec<Option<String>>>,
    // Schedule type, once or cron
    pub schedule_type: ScheduleTypeEnum,
    // If cron, cron expression i.e. * * * * *
    pub cron_expression: Option<String>,
    // Optional notes about job, for UI
    pub notes: Option<String>,
    // Job state, job life cycle, submitted, queued, running, completed, ..
    pub state: JobStateEnum,
    // If failed, error message
    pub error_message: Option<String>,
    // When job was created
    pub created_at: NaiveDateTime,
    // When job was last updated
    pub updated_at: NaiveDateTime,
}

// Display job
impl fmt::Display for Job {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}\nImage: {} ({:?})\nSchedule: {:?}{}\nState: {:?} | Created: {}",
            self.id,
            self.job_name,
            self.image_url,
            self.image_format,
            self.schedule_type,
            match &self.cron_expression {
                Some(expr) => format!(" ({})", expr),
                None => "".into(),
            },
            self.state,
            self.created_at.format("%Y-%m-%d %H:%M")
        )
    }
}

/// Insertable struct, some fields are created dynamically during insert
/// Thus the need for a separate "new" struct
#[derive(Debug, Insertable, Deserialize, Serialize)]
#[diesel(table_name = jobs)]
pub struct NewJob {
    // FK
    pub user_id: i32,
    // Job name
    pub job_name: String,
    // Docker image url, where it is found
    pub image_url: String,
    // Docker image format, either tarball or dockerfile
    pub image_format: ImageFormatEnum,
    // Docker flags, how dockerfile should be ran
    pub docker_flags: Option<Vec<Option<String>>>,
    // Output type, either stdout or files
    pub output_type: OutputTypeEnum,
    // If files, list files from where to collect
    pub output_paths: Option<Vec<Option<String>>>,
    // Schedule type, once or cron
    pub schedule_type: ScheduleTypeEnum,
    // If cron, cron expression i.e. * * * * *
    pub cron_expression: Option<String>,
    // Optional notes about job, for UI
    pub notes: Option<String>,
    // Job state, job life cycle, submitted, queued, running, completed, ..
    pub state: JobStateEnum,
}

/// Assignment of a job to a worker, binds job to specific worker/runner
#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(Job))] // FK: job_id
#[diesel(belongs_to(Worker))] // FK: worker_id
pub struct JobAssignment {
    pub id: i32,
    // FK
    pub job_id: i32,
    // FK
    pub worker_id: i32,
    // When job was assigned
    pub assigned_at: NaiveDateTime,
    // When job was started
    pub started_at: Option<NaiveDateTime>,
    // When job was finished
    pub finished_at: Option<NaiveDateTime>,
}

/// Insertable struct, used since certain fields are generated post insert
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = job_assignments)]
pub struct NewJobAssignment {
    // FK
    pub job_id: i32,
    // FK
    pub worker_id: i32,
}

/// Holds information abuot a job result
#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(Job))] // FK: job_id
pub struct JobResult {
    pub id: i32,
    // FK
    pub job_id: i32,
    // Stdout
    pub stdout: Option<String>,
    // Files
    pub files: Option<Vec<Option<String>>>, // JSON structure
    // When job result was saved
    pub saved_at: NaiveDateTime,
}

/// Insertable struct
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = job_results)]
pub struct NewJobResult {
    pub job_id: i32,
    pub stdout: Option<String>,
    pub files: Option<Vec<Option<String>>>,
}

/// Holds information abuot a job metric, created after a job is assigned worker
#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(Job))] // FK: job_id
#[diesel(belongs_to(Worker))] // FK: worker_id
#[diesel(table_name = job_metrics)]
pub struct JobMetric {
    pub id: i32,
    // FK
    pub job_id: i32,
    // FK
    pub worker_id: i32,
    // Duration
    pub duration_sec: Option<i32>,
    // CPU usage in %
    pub cpu_usage_pct: Option<f32>,
    // Memory usage in MB
    pub mem_usage_mb: Option<f32>,
    // Exit code
    pub exit_code: Option<i32>,
    // When metric was saved
    pub timestamp: NaiveDateTime,
}

/// Insertable struct
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = job_metrics)]
pub struct NewJobMetric {
    // FK
    pub job_id: i32,
    // FK
    pub worker_id: i32,
    // Duration
    pub duration_sec: Option<i32>,
    // CPU usage in %
    pub cpu_usage_pct: Option<f32>,
    // Memory usage in MB
    pub mem_usage_mb: Option<f32>,
    // Exit code
    pub exit_code: Option<i32>,
}
