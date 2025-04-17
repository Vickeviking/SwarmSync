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

/* ==================== Routes WorkerStatus ====================
== CRUD ==
• `POST /worker-status` → create(NewWorkerStatus) -> WorkerStatus
• `GET /worker-status/:id` → find_by_id(id) -> WorkerStatus
• `DELETE /worker-status/:id` → delete_worker_status(id) -> usize

== Lookup & Search ==
• `GET /worker-status/worker/:worker_id` → find_by_worker_id(worker_id) -> Option<WorkerStatus>

== State Update ==
• `PUT /worker-status/:id/status` → update_status(id, status) -> WorkerStatus
• `PUT /worker-status/:id/last-heartbeat` → update_last_heartbeat(id, last_heartbeat) -> WorkerStatus
• `PUT /worker-status/:id/active-job-id` → update_active_job_id(id, active_job_id) -> WorkerStatus
• `PUT /worker-status/:id/uptime` → update_uptime(id, uptime_sec) -> WorkerStatus
• `PUT /worker-status/:id/load-avg` → update_load_avg(id, load_avg) -> WorkerStatus
• `PUT /worker-status/:id/last-error` → update_last_error(id, last_error) -> WorkerStatus

*/
