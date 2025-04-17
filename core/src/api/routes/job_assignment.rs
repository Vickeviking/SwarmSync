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

/* ================================== Routes JobAssignment ==================================

== CRUD ==
• `POST /assignments`        → create(NewJobAssignment) -> JobAssignment
• `GET /assignments/:id`     → find_by_id(id) -> JobAssignment
• `DELETE /assignments/:id`  → delete(id) -> usize

== Lookup & Search ==
• `GET /assignments/by_job/:job_id`             → find_by_job_id(job_id) -> Vec<JobAssignment>
• `GET /assignments/by_worker/:worker_id`       → find_by_worker_id(worker_id) -> Vec<JobAssignment>
• `GET /assignments/lookup/:job_id/:worker_id`  → find_assignment_by_job_and_worker(job_id, worker_id) -> Option<JobAssignment>
• `GET /assignments/by_worker/range`            → find_assignments_for_worker_in_time_range(worker_id, start, end) -> Vec<JobAssignment>

== State Updates ==
• `PATCH /assignments/:id/started`   → update_started_at(id, started_at) -> JobAssignment
• `PATCH /assignments/:id/finished`  → update_finished_at(id, finished_at) -> JobAssignment

== Filtering / Status ==
• `GET /assignments/active` → list_active_assignments() -> Vec<JobAssignment>

*/
