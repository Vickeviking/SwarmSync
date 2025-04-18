use crate::database::models::job::{JobMetric, NewJobMetric};
use crate::database::repositories::{
    JobAssignmentRepository, JobMetricRepository, JobRepository, JobResultRepository,
    LogEntryRepository, UserRepository, WorkerRepository, WorkerStatusRepository,
};
use crate::shared::{enums::system::CoreEvent, SharedResources};

use crate::api::DbConn;
use rocket::http::Status;
use rocket::response::status::{Custom, NoContent};
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, post, routes, Build, Rocket, Route, Shutdown};
use rocket_db_pools::{Connection, Database};
use std::env;
use std::sync::Arc;
use tokio::sync::{broadcast::error::RecvError, mpsc, RwLock};

pub fn routes() -> Vec<Route> {
    routes![
        create_metric,
        get_metric,
        delete_job,
        get_metrics_by_job_id,
        get_metrics_by_worker_id,
        get_most_recent_for_job,
        list_metrics_for_job,
        get_metrics_worker_stream
    ]
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

/* ========== JobMetric ===========
#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(Job))] // FK: job_id
#[diesel(belongs_to(Worker))] // FK: worker_id
#[diesel(table_name = job_metrics)]
pub struct JobMetric {
    pub id: i32,
    pub job_id: i32,
    pub worker_id: i32,
    pub duration_sec: Option<i32>,
    pub cpu_usage_pct: Option<f32>,
    pub mem_usage_mb: Option<f32>,
    pub exit_code: Option<i32>,
    pub timestamp: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = job_metrics)]
pub struct NewJobMetric {
    pub job_id: i32,
    pub worker_id: i32,
    pub duration_sec: Option<i32>,
    pub cpu_usage_pct: Option<f32>,
    pub mem_usage_mb: Option<f32>,
    pub exit_code: Option<i32>,
}
*/

// ===== CRUD =======
#[post("/metrics", format = "json", data = "<new_metric>")]
pub async fn create_metric(
    mut db: Connection<DbConn>,
    new_metric: Json<NewJobMetric>,
) -> Result<Custom<Json<JobMetric>>, Custom<Json<serde_json::Value>>> {
    JobMetricRepository::create(&mut db, new_metric.into_inner())
        .await
        .map(|metric| Custom(Status::Created, Json(metric)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({ "error": e.to_string() })),
            )
        })
}

#[get("/metrics/<id>")]
pub async fn get_metric(
    mut db: Connection<DbConn>,
    id: i32,
) -> Result<Json<JobMetric>, Custom<Json<serde_json::Value>>> {
    JobMetricRepository::find_by_id(&mut db, id)
        .await
        .map(Json)
        .map_err(|e| Custom(Status::NotFound, Json(json!({ "error": e.to_string() }))))
}

#[delete("/metrics/<id>")]
pub async fn delete_job(
    mut db: Connection<DbConn>,
    id: i32,
) -> Result<Status, Custom<Json<serde_json::Value>>> {
    JobMetricRepository::delete(&mut db, id)
        .await
        .map(|_| Status::NoContent)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({ "error": e.to_string() })),
            )
        })
}

// ===== Lookup & Search =====
#[get("/metrics/by_job/<job_id>")]
pub async fn get_metrics_by_job_id(
    mut db: Connection<DbConn>,
    job_id: i32,
) -> Result<Json<Vec<JobMetric>>, Custom<Value>> {
    JobMetricRepository::find_by_job_id(&mut db, job_id)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                json!({ "error": e.to_string() }),
            )
        })
}

#[get("/metrics/by_worker/<worker_id>")]
pub async fn get_metrics_by_worker_id(
    mut db: Connection<DbConn>,
    worker_id: i32,
) -> Result<Json<Vec<JobMetric>>, Custom<Value>> {
    JobMetricRepository::find_by_worker_id(&mut db, worker_id)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                json!({ "error": e.to_string() }),
            )
        })
}

#[get("/metrics/recent/<job_id>")]
pub async fn get_most_recent_for_job(
    mut db: Connection<DbConn>,
    job_id: i32,
) -> Result<Json<Option<JobMetric>>, Custom<Value>> {
    JobMetricRepository::get_most_recent_for_job(&mut db, job_id)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                json!({ "error": e.to_string() }),
            )
        })
}

#[get("/metrics/chronological/<job_id>")]
pub async fn list_metrics_for_job(
    mut db: Connection<DbConn>,
    job_id: i32,
) -> Result<Json<Vec<JobMetric>>, Custom<Value>> {
    JobMetricRepository::list_metrics_for_job(&mut db, job_id)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                json!({ "error": e.to_string() }),
            )
        })
}

#[get("/metrics/worker_stream/<worker_id>")]
pub async fn get_metrics_worker_stream(
    mut db: Connection<DbConn>,
    worker_id: i32,
) -> Result<Json<Vec<JobMetric>>, Custom<Value>> {
    JobMetricRepository::get_metrics_by_worker(&mut db, worker_id)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                json!({ "error": e.to_string() }),
            )
        })
}
