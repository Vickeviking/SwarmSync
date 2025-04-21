// ********** FILE CONTENT **********
//  Models for:
//      Job, JobAssignment, JobResult, JobMetrics
//
// ***********************************

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::models::{user::User, worker::Worker};
use crate::database::schema::*;
use crate::shared::enums::{
    image_format::ImageFormatEnum, job::JobStateEnum, output::OutputTypeEnum,
    schedule::ScheduleTypeEnum,
};

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(User))] // FK: user_id
pub struct Job {
    pub id: i32,
    pub user_id: i32,
    pub job_name: String,
    pub image_url: String,
    pub image_format: ImageFormatEnum,
    pub docker_flags: Option<Vec<Option<String>>>,
    pub output_type: OutputTypeEnum,
    pub output_paths: Option<Vec<Option<String>>>,
    pub schedule_type: ScheduleTypeEnum,
    pub cron_expression: Option<String>,
    pub notes: Option<String>,
    pub state: JobStateEnum,
    pub error_message: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Deserialize, Serialize)]
#[diesel(table_name = jobs)]
pub struct NewJob {
    pub user_id: i32,
    pub job_name: String,
    pub image_url: String,
    pub image_format: ImageFormatEnum,
    pub docker_flags: Option<Vec<Option<String>>>,
    pub output_type: OutputTypeEnum,
    pub output_paths: Option<Vec<Option<String>>>,
    pub schedule_type: ScheduleTypeEnum,
    pub cron_expression: Option<String>,
    pub notes: Option<String>,
    pub state: JobStateEnum,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(Job))] // FK: job_id
#[diesel(belongs_to(Worker))] // FK: worker_id
pub struct JobAssignment {
    pub id: i32,
    pub job_id: i32,
    pub worker_id: i32,
    pub assigned_at: NaiveDateTime,
    pub started_at: Option<NaiveDateTime>,
    pub finished_at: Option<NaiveDateTime>,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = job_assignments)]
pub struct NewJobAssignment {
    pub job_id: i32,
    pub worker_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(Job))] // FK: job_id
pub struct JobResult {
    pub id: i32,
    pub job_id: i32,
    pub stdout: Option<String>,
    pub files: Option<Vec<Option<String>>>, // JSON structure
    pub saved_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = job_results)]
pub struct NewJobResult {
    pub job_id: i32,
    pub stdout: Option<String>,
    pub files: Option<Vec<Option<String>>>,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(Job))] // FK: job_id
#[diesel(belongs_to(Worker))] // FK: worker_id
#[diesel(table_name = job_metrics)]
pub struct JobMetric {
    pub id: i32,
    pub job_id: i32,
    pub worker_id: i32,
    pub duration_sec: Option<i32>,
    pub cpu_usage_pct: Option<f32>,
    pub mem_usage_mb: Option<f32>,
    pub exit_code: Option<i32>,
    pub timestamp: NaiveDateTime,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = job_metrics)]
pub struct NewJobMetric {
    pub job_id: i32,
    pub worker_id: i32,
    pub duration_sec: Option<i32>,
    pub cpu_usage_pct: Option<f32>,
    pub mem_usage_mb: Option<f32>,
    pub exit_code: Option<i32>,
}
