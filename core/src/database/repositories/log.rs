/*
========================
🛠️ Supported Actions Log
========================

== CRUD ==
• `POST /logs`             → create(NewDBLogEntry) -> LogEntry
• `GET /logs/:id`          → find_by_id(id) -> LogEntry
• `DELETE /logs/:id`       → delete(id) -> usize
• `PUT /logs/:id`          → update(id, LogEntry) -> LogEntry

== Lookup & Search ==
• `GET /logs/search/level?q=INFO`       → search_by_level(query) -> Vec<LogEntry>
• `GET /logs/search/module?q=Scheduler` → find_logs_by_module(module: SystemModuleEnum)
• `GET /logs/search/action?q=dispatch`  → search_by_action(query)
• `GET /logs/job/:job_id`               → list_logs_by_job_id (not implemented yet, implied)
• `GET /logs/expiring`                  → list_logs_by_expiry (not implemented yet, implied)
• `GET /logs?page=x&limit=y`            → list_all(limit, offset)

== Field Updates ==
• `PATCH /logs/:id/msg`       → update_custom_msg(id, msg) -> LogEntry (not implemented yet, implied)
• `PATCH /logs/:id/ttl`       → update_expires_at(id, new_time) -> LogEntry (not implemented yet, implied)

== Existence Checks ==
• `HEAD /logs/exists?action=foo`  → exists_by_action(action) -> bool
• `HEAD /logs/exists?level=info`  → exists_by_level(level) -> bool
*/

use crate::database::models::log::{DBLogEntry, NewDBLogEntry};
use crate::database::schema::logs;
use crate::shared::enums::system::SystemModuleEnum;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

pub struct LogEntryRepository;

impl LogEntryRepository {
    // Find log by ID
    pub async fn find_by_id(c: &mut AsyncPgConnection, id: i32) -> QueryResult<DBLogEntry> {
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
    ) -> QueryResult<DBLogEntry> {
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

    // Search logs by action
    pub async fn search_by_action(
        c: &mut AsyncPgConnection,
        query: &str,
    ) -> QueryResult<Vec<DBLogEntry>> {
        logs::table
            .filter(logs::action.ilike(format!("%{}%", query)))
            .load(c)
            .await
    }

    pub async fn search_by_level(
        c: &mut AsyncPgConnection,
        query: &str,
    ) -> QueryResult<Vec<DBLogEntry>> {
        logs::table
            .filter(logs::level.ilike(format!("%{}%", query)))
            .load(c)
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

    // Get logs with specific module
    pub async fn find_logs_by_module(
        c: &mut AsyncPgConnection,
        module: SystemModuleEnum,
    ) -> QueryResult<Vec<DBLogEntry>> {
        logs::table
            .filter(logs::module.eq(module))
            .load(c)
            .await
            .map(|db_logs: Vec<DBLogEntry>| {
                db_logs.into_iter().map(|db_log| db_log.into()).collect()
            })
    }
}
