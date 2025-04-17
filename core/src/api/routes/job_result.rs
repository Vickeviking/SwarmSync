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
============================== Routes JobResult ==============================

== CRUD ==
• `POST /results`        → create(NewJobResult) -> JobResult
• `GET /results/:id`     → find_by_id(id) -> JobResult
• `DELETE /results/:id`  → delete(id) -> usize

== Lookup & Search ==
• `GET /results/job/:job_id`         → find_by_job_id(job_id) -> Vec<JobResult>
• `GET /results/list/:job_id`        → list_results_for_job(job_id) -> Vec<JobResult>
• `GET /results/recent/:job_id`      → get_most_recent_for_job(job_id) -> Option<JobResult>

== Field Updates ==
• `PATCH /results/:id/stdout`        → update_stdout(id, new_stdout) -> JobResult
• `PATCH /results/:id/files`         → update_files(id, new_files) -> JobResult
*/
