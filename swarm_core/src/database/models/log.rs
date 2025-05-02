// ********** FILE CONTENT **********
//  Models for:
//      Log
//
// ***********************************

use chrono::NaiveDateTime;
use diesel::{prelude::*, Insertable};
use serde::{Deserialize, Serialize};

use crate::database::schema::*;
use crate::shared::enums::{log::LogActionEnum, log::LogLevelEnum, system::SystemModuleEnum};

// Used in the application
pub struct LogEntry {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub level: LogLevelEnum,
    pub module: SystemModuleEnum,
    pub action: LogActionEnum,
    pub expires_at: NaiveDateTime,

    // Embed the payloads directly
    pub client_connected_payload: Option<ClientConnectedPayload>,
    pub job_submitted_payload: Option<JobSubmittedPayload>,
    pub job_completed_payload: Option<JobCompletedPayload>,
    pub custom_msg: Option<String>,
}

pub struct ClientConnectedPayload {
    pub ip: String,
    pub username: String,
}

pub struct JobSubmittedPayload {
    pub job_id: i32,
    pub from_module: SystemModuleEnum,
    pub to_module: SystemModuleEnum,
}

pub struct JobCompletedPayload {
    pub job_id: i32,
    pub success: bool,
}

// ====== DATABASE STORED STRUCTS ====

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = logs)]
pub struct DBLogEntry {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub level: LogLevelEnum,
    pub module: SystemModuleEnum,
    pub action: LogActionEnum,
    pub expires_at: NaiveDateTime,

    // Optional fields for the foreign keys to payload tables
    pub client_connected_ip: Option<String>, // Nullable IP field
    pub client_connected_username: Option<String>, // Nullable Username field

    pub job_submitted_job_id: Option<i32>, // Nullable Job ID
    pub job_submitted_from_module: Option<SystemModuleEnum>, // Nullable From module
    pub job_submitted_to_module: Option<SystemModuleEnum>, // Nullable To module

    pub job_completed_job_id: Option<i32>,   // Nullable Job ID
    pub job_completed_success: Option<bool>, // Nullable Success flag

    pub custom_msg: Option<String>, // Nullable custom message
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = logs)]
pub struct NewDBLogEntry {
    pub level: LogLevelEnum,
    pub module: SystemModuleEnum,
    pub action: LogActionEnum,
    pub expires_at: NaiveDateTime,

    // Optional fields for the foreign keys to payload tables
    pub client_connected_ip: Option<String>, // Nullable IP field
    pub client_connected_username: Option<String>, // Nullable Username field

    pub job_submitted_job_id: Option<i32>, // Nullable Job ID
    pub job_submitted_from_module: Option<SystemModuleEnum>, // Nullable From module
    pub job_submitted_to_module: Option<SystemModuleEnum>, // Nullable To module

    pub job_completed_job_id: Option<i32>,   // Nullable Job ID
    pub job_completed_success: Option<bool>, // Nullable Success flag

    pub custom_msg: Option<String>, // Nullable custom message
}

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
