use crate::enums::{log::LogActionEnum, log::LogLevelEnum, system::SystemModuleEnum};
/**
 ==== Supported Operations ====

 == CRUD
- create
- find_by_id
- delete

 == Lookup & Search
- find_by_level
- find_by_module
- find_by_action
- list_logs_by_expiry
- search_logs_by_message
- list_logs_by_job_id

 == Field Updates
- update_custom_msg
- update_expires_at
*/
use crate::models::log::{DBLogEntry, LogEntry, NewDBLogEntry};
use crate::schema::logs;
use diesel::dsl::now;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

pub struct LogEntryRepository;

impl LogEntryRepository {
    // Find log by ID
    pub async fn find_by_id(c: &mut AsyncPgConnection, id: i32) -> QueryResult<LogEntry> {
        logs::table
            .find(id)
            .get_result(c)
            .await
            .map(|db_log: DBLogEntry| db_log.into())
    }

    // Create a new log entry
    pub async fn create(
        c: &mut AsyncPgConnection,
        new_log_entry: NewDBLogEntry,
    ) -> QueryResult<LogEntry> {
        diesel::insert_into(logs::table)
            .values(new_log_entry)
            .get_result(c)
            .await
            .map(|db_log: DBLogEntry| db_log.into())
    }

    // Delete a log entry by ID
    pub async fn delete(c: &mut AsyncPgConnection, id: i32) -> QueryResult<usize> {
        diesel::delete(logs::table.find(id)).execute(c).await
    }

    // Update an existing log entry by ID
    pub async fn update(
        c: &mut AsyncPgConnection,
        id: i32,
        updated_log_entry: LogEntry,
    ) -> QueryResult<LogEntry> {
        diesel::update(logs::table.find(id))
            .set((
                logs::level.eq(updated_log_entry.level),
                logs::module.eq(updated_log_entry.module),
                logs::action.eq(updated_log_entry.action),
                logs::expires_at.eq(updated_log_entry.expires_at),
                logs::client_connected_ip.eq(updated_log_entry
                    .client_connected_payload
                    .as_ref()
                    .map(|p| p.ip.clone())),
                logs::client_connected_username.eq(updated_log_entry
                    .client_connected_payload
                    .as_ref()
                    .map(|p| p.username.clone())),
                logs::job_submitted_job_id.eq(updated_log_entry
                    .job_submitted_payload
                    .as_ref()
                    .map(|p| p.job_id)),
                logs::job_submitted_from_module.eq(updated_log_entry
                    .job_submitted_payload
                    .as_ref()
                    .map(|p| p.from_module.clone())),
                logs::job_submitted_to_module.eq(updated_log_entry
                    .job_submitted_payload
                    .as_ref()
                    .map(|p| p.to_module.clone())),
                logs::job_completed_job_id.eq(updated_log_entry
                    .job_completed_payload
                    .as_ref()
                    .map(|p| p.job_id)),
                logs::job_completed_success.eq(updated_log_entry
                    .job_completed_payload
                    .as_ref()
                    .map(|p| p.success)),
                logs::custom_msg.eq(updated_log_entry.custom_msg),
            ))
            .get_result(c)
            .await
            .map(|db_log: DBLogEntry| db_log.into())
    }

    // Search logs by action
    pub async fn search_by_action(
        c: &mut AsyncPgConnection,
        query: &str,
    ) -> QueryResult<Vec<LogEntry>> {
        logs::table
            .filter(logs::action.ilike(format!("%{}%", query)))
            .load(c)
            .await
            .map(|db_logs: Vec<DBLogEntry>| {
                db_logs.into_iter().map(|db_log| db_log.into()).collect()
            })
    }

    // Search logs by level
    pub async fn search_by_level(
        c: &mut AsyncPgConnection,
        query: &str,
    ) -> QueryResult<Vec<LogEntry>> {
        logs::table
            .filter(logs::level.ilike(format!("%{}%", query)))
            .load(c)
            .await
            .map(|db_logs: Vec<DBLogEntry>| {
                db_logs.into_iter().map(|db_log| db_log.into()).collect()
            })
    }

    // List all logs with pagination
    pub async fn list_all(
        c: &mut AsyncPgConnection,
        limit: i64,
        offset: i64,
    ) -> QueryResult<Vec<LogEntry>> {
        logs::table
            .limit(limit)
            .offset(offset)
            .load(c)
            .await
            .map(|db_logs: Vec<DBLogEntry>| {
                db_logs.into_iter().map(|db_log| db_log.into()).collect()
            })
    }

    // Check if a log with the specific action exists
    pub async fn exists_by_action(c: &mut AsyncPgConnection, action: &str) -> QueryResult<bool> {
        let count: i64 = logs::table
            .filter(logs::action.eq(action))
            .count()
            .get_result(c)
            .await?;
        Ok(count > 0)
    }

    // Check if a log with a specific level exists
    pub async fn exists_by_level(c: &mut AsyncPgConnection, level: &str) -> QueryResult<bool> {
        let count: i64 = logs::table
            .filter(logs::level.eq(level))
            .count()
            .get_result(c)
            .await?;
        Ok(count > 0)
    }

    // Get logs with specific module
    pub async fn find_logs_by_module(
        c: &mut AsyncPgConnection,
        module: SystemModuleEnum,
    ) -> QueryResult<Vec<LogEntry>> {
        logs::table
            .filter(logs::module.eq(module))
            .load(c)
            .await
            .map(|db_logs: Vec<DBLogEntry>| {
                db_logs.into_iter().map(|db_log| db_log.into()).collect()
            })
    }
}
