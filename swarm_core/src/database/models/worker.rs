// ********** FILE CONTENT **********
//  Models for:
//      Worker, WorkerStatus
//
// ***********************************

use std::fmt;

use crate::database::models::user::User;
use crate::database::schema::{worker_status, workers};
use crate::shared::enums::workers::{OSEnum, WorkerStatusEnum};
use chrono::NaiveDateTime;
use diesel::{Associations, Identifiable, Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(User))] // FK: user_id
#[diesel(table_name = workers)]
pub struct Worker {
    pub id: i32,
    pub user_id: i32,
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

impl fmt::Display for Worker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}\nOS: {:?} | Arch: {}\nIP: {} | SSH: {}@{}\nDocker: {}\nLast seen: {}",
            self.id,
            self.label,
            self.os,
            self.arch,
            self.ip_address,
            self.ssh_user,
            self.hostname,
            self.docker_version,
            self.last_seen_at
                .map(|ts| ts.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Never".into())
        )
    }
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = workers)]
pub struct NewWorker {
    pub user_id: i32,
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
