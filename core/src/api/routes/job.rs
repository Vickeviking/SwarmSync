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

/*
======================== Routes Job ========================

== CRUD ==
• `POST /jobs`          → create(NewJob) -> Job
• `GET /jobs/:id`       → find_by_id(id) -> Job
• `PATCH /jobs/:id`     → update(id, Job) -> Job
• `DELETE /jobs/:id`    → delete(id) -> usize

== Lookup & Search ==
• `GET /jobs/search`    → search_by_job_name(user_id, query) -> Vec<Job>
• `GET /jobs/name/:str` → find_by_name(user_id, name) -> Option<Job>
• `GET /jobs/by_admin`  → list_by_admin(user_id, limit, offset) -> Vec<Job>
• `GET /jobs/state/:st` → list_by_state(state) -> Vec<Job>
• `GET /jobs/recent`    → get_recent_jobs(limit) -> Vec<Job>
• `GET /jobs/failed`    → get_failed_jobs(limit) -> Vec<Job>

== State Transitions ==
• `PATCH /jobs/:id/running`   → mark_running(id) -> Job
• `PATCH /jobs/:id/succeeded` → mark_succeeded(id) -> Job
• `PATCH /jobs/:id/failed`    → mark_failed(id, msg) -> Job

== Scheduling & Readiness ==
• `GET /jobs/scheduled`       → list_scheduled_jobs() -> Vec<Job>
• `GET /jobs/cron_due`        → list_due_cron_jobs(current_time) -> Vec<Job>
• `GET /jobs/ready`           → list_one_time_jobs_ready() -> Vec<Job>

== Aggregation & Stats ==
• `GET /jobs/stats/admins`    → get_job_counts_per_admin() -> Vec<(admin_id, job_count)>

== Assignment-related ==
• `GET /jobs/active/:worker`  → get_active_jobs_for_worker(worker_id) -> Vec<Job>
• `GET /jobs/assigned/:worker`→ find_jobs_assigned_to_worker(worker_id) -> Vec<Job>
• `GET /jobs/unassigned`      → list_jobs_with_no_assignment() -> Vec<Job>

*/

#[rocket::post("/jobs")]
fn create_job() {}
