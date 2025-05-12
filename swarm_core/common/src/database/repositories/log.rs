use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::database::models::log::{DBLogEntry, NewDBLogEntry};
use crate::database::schema::logs;
use crate::enums::log::{LogActionEnum, LogLevelEnum};
use crate::enums::system::SystemModuleEnum;

pub struct LogEntryRepository;

impl LogEntryRepository {
    // Find log by ID
    pub async fn find_by_id(c: &mut AsyncPgConnection, id: i32) -> QueryResult<DBLogEntry> {
        logs::table
            .find(id)
            .get_result(c)
            .await
            .map(|db_log: DBLogEntry| db_log)
    }

    // Create a new log entry
    pub async fn create(
        c: &mut AsyncPgConnection,
        new_log_entry: NewDBLogEntry,
    ) -> QueryResult<DBLogEntry> {
        diesel::insert_into(logs::table)
            .values(new_log_entry)
            .get_result(c)
            .await
            .map(|db_log: DBLogEntry| db_log)
    }

    // Delete a log entry by ID
    pub async fn delete(c: &mut AsyncPgConnection, id: i32) -> QueryResult<usize> {
        diesel::delete(logs::table.find(id)).execute(c).await
    }

    // Update an existing log entry by ID
    pub async fn update(
        c: &mut AsyncPgConnection,
        id: i32,
        updated: DBLogEntry,
    ) -> QueryResult<DBLogEntry> {
        diesel::update(logs::table.find(id))
            .set((
                logs::level.eq(updated.level),
                logs::module.eq(updated.module),
                logs::action.eq(updated.action),
                logs::expires_at.eq(updated.expires_at),
                logs::client_connected_ip.eq(updated.client_connected_ip),
                logs::client_connected_username.eq(updated.client_connected_username),
                logs::job_submitted_job_id.eq(updated.job_submitted_job_id),
                logs::job_submitted_from_module.eq(updated.job_submitted_from_module),
                logs::job_submitted_to_module.eq(updated.job_submitted_to_module),
                logs::job_completed_job_id.eq(updated.job_completed_job_id),
                logs::job_completed_success.eq(updated.job_completed_success),
                logs::custom_msg.eq(updated.custom_msg),
            ))
            .get_result::<DBLogEntry>(c)
            .await
    }

    pub async fn list_all(
        c: &mut AsyncPgConnection,
        limit: i64,
        offset: i64,
    ) -> QueryResult<Vec<DBLogEntry>> {
        logs::table.limit(limit).offset(offset).load(c).await
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

    /// List logs whose `action` equals the given enum variant, with pagination
    pub async fn list_by_action_exact(
        c: &mut AsyncPgConnection,
        action: LogActionEnum,
        limit: i64,
        offset: i64,
    ) -> QueryResult<Vec<DBLogEntry>> {
        logs::table
            .filter(logs::action.eq(action))
            .limit(limit)
            .offset(offset)
            .load(c)
            .await
    }

    /// List logs whose `level` equals the given enum variant, with pagination
    pub async fn list_by_level_exact(
        c: &mut AsyncPgConnection,
        level: LogLevelEnum,
        limit: i64,
        offset: i64,
    ) -> QueryResult<Vec<DBLogEntry>> {
        logs::table
            .filter(logs::level.eq(level))
            .limit(limit)
            .offset(offset)
            .load(c)
            .await
    }

    /// List logs for a specific module, with pagination
    pub async fn find_logs_by_module(
        c: &mut AsyncPgConnection,
        module: SystemModuleEnum,
        limit: i64,
        offset: i64,
    ) -> QueryResult<Vec<DBLogEntry>> {
        logs::table
            .filter(logs::module.eq(module))
            .limit(limit)
            .offset(offset)
            .load(c)
            .await
    }
}
