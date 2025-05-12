// ********** FILE CONTENT **********
//  Models for:
//      Worker, WorkerStatus
//
// ***********************************

use std::fmt;

use chrono::NaiveDateTime;
use diesel::{Associations, Identifiable, Insertable, Queryable};
use serde::{Deserialize, Serialize};

use crate::database::models::user::User;
use crate::database::schema::{worker_status, workers};
use crate::enums::workers::{OSEnum, WorkerStatusEnum};

/// Worker model, represents a connected swarm worker.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(User))] // FK: user_id
#[diesel(table_name = workers)]
pub struct Worker {
    pub id: i32,
    // FK, owning useraccount
    pub user_id: i32,
    // worker label, printed as worker name
    pub label: String,
    // worker IP
    pub ip_address: String,
    // worker hostname
    pub hostname: String,
    // worker SSH user, not yet used
    pub ssh_user: String,
    // worker SSH key, not yet used
    pub ssh_key: String,
    // worker docker version
    pub docker_version: String,
    // worker architecture
    pub arch: String,
    // worker OS
    pub os: OSEnum,
    // worker tags, such as "arm64", "amd64", not yet used
    pub tags: Option<Vec<Option<String>>>,
    pub created_at: NaiveDateTime,
    // Last seen timestamp
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

// Insertable
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = workers)]
pub struct NewWorker {
    pub user_id: i32,
    // worker label
    pub label: String,
    // worker IP
    pub ip_address: String,
    // worker hostname
    pub hostname: String,
    // worker SSH user, not yet used
    pub ssh_user: String,
    // worker SSH key, not yet used
    pub ssh_key: String,
    // worker docker version
    pub docker_version: String,
    // worker architecture
    pub arch: String,
    // worker OS
    pub os: OSEnum,
    // worker tags, such as "arm64", "amd64", not yet used
    pub tags: Option<Vec<Option<String>>>,
}

// Connects a worker/runner to a status.
// only one status per worker
#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(Worker))]
#[diesel(table_name = worker_status)]
pub struct WorkerStatus {
    pub id: i32,
    // FK
    pub worker_id: i32,
    // worker status. Busy, Idle, Offline, Unreachable
    pub status: WorkerStatusEnum,
    // Timestamp of the last heartbeat, heartbeat used to update core about online status and
    // metrics
    pub last_heartbeat: Option<NaiveDateTime>,
    // ID of the currently running job
    pub active_job_id: Option<i32>,
    // Uptime in seconds
    pub uptime_sec: Option<i32>,
    // load average, 1 min, 5 min, 15 min, not yet used
    pub load_avg: Option<Vec<Option<f32>>>,
    // last error the worker encountered
    pub last_error: Option<String>,
    // Timestamps
    pub updated_at: NaiveDateTime,
}

// Insertable
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = worker_status)]
pub struct NewWorkerStatus {
    // FK
    pub worker_id: i32,
    // worker status
    pub status: WorkerStatusEnum,
    // Timestamp of the last heartbeat
    pub last_heartbeat: Option<NaiveDateTime>,
    // ID of the currently running job
    pub active_job_id: Option<i32>,
    // Uptime in seconds
    pub uptime_sec: Option<i32>,
    // load average
    pub load_avg: Option<Vec<Option<f32>>>,
    // last error
    pub last_error: Option<String>,
}
