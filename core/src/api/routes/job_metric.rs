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
============================== Routes JobMetric ==============================

== CRUD ==
• `POST /metrics`        → create(NewJobMetric) -> JobMetric
• `GET /metrics/:id`     → find_by_id(id) -> JobMetric
• `DELETE /metrics/:id`  → delete(id) -> usize

== Lookup & Search ==
• `GET /metrics/by_job/:job_id`          → find_by_job_id(job_id) -> Vec<JobMetric>
• `GET /metrics/by_worker/:worker_id`    → find_by_worker_id(worker_id) -> Vec<JobMetric>
• `GET /metrics/recent/:job_id`          → get_most_recent_for_job(job_id) -> Option<JobMetric>
• `GET /metrics/chronological/:job_id`   → list_metrics_for_job(job_id) -> Vec<JobMetric>
• `GET /metrics/worker_stream/:worker_id` → get_metrics_by_worker(worker_id) -> Vec<JobMetric>
*/
