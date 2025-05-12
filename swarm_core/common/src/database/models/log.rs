use chrono::NaiveDateTime;
use diesel::{prelude::*, Insertable};
use serde::{Deserialize, Serialize};

use crate::database::schema::*;
use crate::enums::{log::LogActionEnum, log::LogLevelEnum, system::SystemModuleEnum};

/// The LogEntry struct is the in-memory representation of the database model
/// In-memory variant needed since the embedded payloads are not serializable?
pub struct LogEntry {
    pub id: i32,
    pub created_at: NaiveDateTime,
    // Level, used to filter, and automatic cleanup timer
    pub level: LogLevelEnum,
    // Module log created from, used to filter
    pub module: SystemModuleEnum,
    // Action describing the log, used to filter, custom action exists
    pub action: LogActionEnum,
    pub expires_at: NaiveDateTime,

    // Payload can be derived from the action: client_connected, job_submitted, job_completed or custom
    // Embed client connected payload, if log revolves a client
    pub client_connected_payload: Option<ClientConnectedPayload>,
    // Embeds job submitted payload if log revolves a job
    pub job_submitted_payload: Option<JobSubmittedPayload>,
    // Embeds job completed payload if log revolves a job
    pub job_completed_payload: Option<JobCompletedPayload>,
    // Optional custom message, used for custom actions
    pub custom_msg: Option<String>,
}

/// Client connected payload
pub struct ClientConnectedPayload {
    pub ip: String,
    // username of client connecting
    pub username: String,
}

/// Job submitted payload
pub struct JobSubmittedPayload {
    pub job_id: i32,
    // From and to module
    pub from_module: SystemModuleEnum,
    pub to_module: SystemModuleEnum,
}

/// Job completed payload
pub struct JobCompletedPayload {
    pub job_id: i32,
    // Whether job completed successfully
    pub success: bool,
}

// ====== DATABASE STORED STRUCTS ====

/// The DBLogEntry struct is the database model of the LogEntry
#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = logs)]
pub struct DBLogEntry {
    pub id: i32,
    pub created_at: NaiveDateTime,
    // Level, used to filter, and automatic cleanup timer
    pub level: LogLevelEnum,
    // Module log created from, used to filter
    pub module: SystemModuleEnum,
    // Action describing the log, used to filter, custom action exists
    pub action: LogActionEnum,
    pub expires_at: NaiveDateTime,

    // Optional fields for the foreign keys to payload tables
    // These payloads are summed up into structs inside the in-memory LogEntry variant
    pub client_connected_ip: Option<String>, // Nullable IP field
    pub client_connected_username: Option<String>, // Nullable Username field

    pub job_submitted_job_id: Option<i32>, // Nullable Job ID
    pub job_submitted_from_module: Option<SystemModuleEnum>, // Nullable From module
    pub job_submitted_to_module: Option<SystemModuleEnum>, // Nullable To module

    pub job_completed_job_id: Option<i32>,   // Nullable Job ID
    pub job_completed_success: Option<bool>, // Nullable Success flag

    pub custom_msg: Option<String>, // Nullable custom message
}

// Insertable DBLogEntry
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = logs)]
pub struct NewDBLogEntry {
    pub level: LogLevelEnum,
    // Module log created from
    pub module: SystemModuleEnum,
    // Action describing the log
    pub action: LogActionEnum,
    pub expires_at: NaiveDateTime,

    // Optional fields for the foreign keys to payload tables
    // These payloads are summed up into structs inside the in-memory LogEntry variant
    pub client_connected_ip: Option<String>, // Nullable IP field
    pub client_connected_username: Option<String>, // Nullable Username field

    pub job_submitted_job_id: Option<i32>, // Nullable Job ID
    pub job_submitted_from_module: Option<SystemModuleEnum>, // Nullable From module
    pub job_submitted_to_module: Option<SystemModuleEnum>, // Nullable To module

    pub job_completed_job_id: Option<i32>,   // Nullable Job ID
    pub job_completed_success: Option<bool>, // Nullable Success flag

    pub custom_msg: Option<String>, // Nullable custom message
}

// Convert from in-memory LogEntry to DBLogEntry
impl From<LogEntry> for NewDBLogEntry {
    fn from(log: LogEntry) -> Self {
        // Convert LogEntry (with embedded payloads) to DBLogEntry (raw database model)
        NewDBLogEntry {
            level: log.level,
            module: log.module,
            action: log.action,
            expires_at: log.expires_at,
            client_connected_ip: log.client_connected_payload.as_ref().map(|p| p.ip.clone()),
            client_connected_username: log
                .client_connected_payload
                .as_ref()
                .map(|p| p.username.clone()),
            job_submitted_job_id: log.job_submitted_payload.as_ref().map(|p| p.job_id),
            job_submitted_from_module: log
                .job_submitted_payload
                .as_ref()
                .map(|p| p.from_module.clone()),
            job_submitted_to_module: log
                .job_submitted_payload
                .as_ref()
                .map(|p| p.to_module.clone()),
            job_completed_job_id: log.job_completed_payload.as_ref().map(|p| p.job_id),
            job_completed_success: log.job_completed_payload.as_ref().map(|p| p.success),
            custom_msg: log.custom_msg,
        }
    }
}

// Convert from DBLogEntry to in-memory LogEntry
impl From<DBLogEntry> for LogEntry {
    fn from(db: DBLogEntry) -> Self {
        let client_connected_payload = match (db.client_connected_ip, db.client_connected_username)
        {
            (Some(ip), Some(username)) => Some(ClientConnectedPayload { ip, username }),
            _ => None,
        };

        let job_submitted_payload = match (
            db.job_submitted_job_id,
            db.job_submitted_from_module,
            db.job_submitted_to_module,
        ) {
            (Some(job_id), Some(from_module), Some(to_module)) => Some(JobSubmittedPayload {
                job_id,
                from_module,
                to_module,
            }),
            _ => None,
        };

        let job_completed_payload = match (db.job_completed_job_id, db.job_completed_success) {
            (Some(job_id), Some(success)) => Some(JobCompletedPayload { job_id, success }),
            _ => None,
        };

        LogEntry {
            id: db.id,
            created_at: db.created_at,
            level: db.level,
            module: db.module,
            action: db.action,
            expires_at: db.expires_at,
            client_connected_payload,
            job_submitted_payload,
            job_completed_payload,
            custom_msg: db.custom_msg,
        }
    }
}

// Convert from in-memory LogEntry to DBLogEntry
impl From<&LogEntry> for NewDBLogEntry {
    fn from(log: &LogEntry) -> Self {
        NewDBLogEntry {
            level: log.level.clone(),
            module: log.module.clone(),
            action: log.action.clone(),
            expires_at: log.expires_at,

            client_connected_ip: log.client_connected_payload.as_ref().map(|p| p.ip.clone()),
            client_connected_username: log
                .client_connected_payload
                .as_ref()
                .map(|p| p.username.clone()),

            job_submitted_job_id: log.job_submitted_payload.as_ref().map(|p| p.job_id),
            job_submitted_from_module: log
                .job_submitted_payload
                .as_ref()
                .map(|p| p.from_module.clone()),
            job_submitted_to_module: log
                .job_submitted_payload
                .as_ref()
                .map(|p| p.to_module.clone()),

            job_completed_job_id: log.job_completed_payload.as_ref().map(|p| p.job_id),
            job_completed_success: log.job_completed_payload.as_ref().map(|p| p.success),

            custom_msg: log.custom_msg.clone(),
        }
    }
}
