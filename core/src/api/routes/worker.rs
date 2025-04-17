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

/* ==================== Routes Worker ====================

== CRUD ==
• `POST /workers` → create(NewWorker) -> Worker
• `GET /workers/:id` → find_by_id(id) -> Worker
• `DELETE /workers/:id` → delete_worker(id) -> usize

== Lookup & Search ==
• `GET /workers/admin/:admin_id` → find_by_admin_id(user_id) -> Vec<Worker>
• `GET /workers/label/:label` → find_by_label(label) -> Option<Worker>
• `GET /workers/ip/:ip_address` → find_by_ip_address(ip_address) -> Option<Worker>
• `GET /workers/admin/:admin_id/list` → list_workers_by_admin(user_id, limit, offset) -> Vec<Worker>

== State Update ==
• `PUT /workers/:id/last-seen` → update_last_seen_at(id, last_seen_at) -> Worker

*/
