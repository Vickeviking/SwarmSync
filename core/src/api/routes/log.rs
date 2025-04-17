use crate::database::repositories::{
    JobAssignmentRepository, JobMetricRepository, JobRepository, JobResultRepository,
    LogEntryRepository, UserRepository, WorkerRepository, WorkerStatusRepository,
};
use crate::shared::{enums::system::CoreEvent, SharedResources};

use rocket::{routes, Build, Rocket, Route, Shutdown};
use rocket_db_pools::Database;
use std::env;
use std::sync::Arc;
use tokio::sync::{broadcast::error::RecvError, mpsc, RwLock};

pub fn routes() -> Vec<Route> {
    routes![]
}

/*======================== Routes Logs ========================

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
