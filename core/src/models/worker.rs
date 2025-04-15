use crate::enums::workers::{OSEnum, WorkerStatusEnum};
use crate::models::admin::Admin;
use crate::schema::{worker_status, workers};
use chrono::NaiveDateTime;
use diesel::{Associations, Identifiable, Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(Admin))] // FK: admin_id
#[diesel(table_name = workers)]
pub struct Worker {
    pub id: i32,
    pub admin_id: i32,
    pub label: String,
    pub ip_address: String,
    pub hostname: String,
    pub ssh_user: String,
    pub ssh_key: String,
    pub docker_version: String,
    pub arch: String,
    pub os: OSEnum,
    pub tags: Option<Vec<Option<String>>>,
    pub created_at: NaiveDateTime,
    pub last_seen_at: Option<NaiveDateTime>,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = workers)]
pub struct NewWorker {
    pub admin_id: i32,
    pub label: String,
    pub ip_address: String,
    pub hostname: String,
    pub ssh_user: String,
    pub ssh_key: String,
    pub docker_version: String,
    pub arch: String,
    pub os: OSEnum,
    pub tags: Option<Vec<Option<String>>>,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(Worker))]
#[diesel(table_name = worker_status)]
pub struct WorkerStatus {
    pub id: i32,
    pub worker_id: i32,
    pub status: WorkerStatusEnum,
    pub last_heartbeat: Option<NaiveDateTime>,
    pub active_job_id: Option<i32>,
    pub uptime_sec: Option<i32>,
    pub load_avg: Option<Vec<Option<f32>>>,
    pub last_error: Option<String>,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = worker_status)]
pub struct NewWorkerStatus {
    pub worker_id: i32,
    pub status: WorkerStatusEnum,
    pub last_heartbeat: Option<NaiveDateTime>,
    pub active_job_id: Option<i32>,
    pub uptime_sec: Option<i32>,
    pub load_avg: Option<Vec<Option<f32>>>,
    pub last_error: Option<String>,
}
