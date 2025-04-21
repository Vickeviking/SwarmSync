use crate::database::models::job::{JobMetric, NewJobMetric};
use crate::database::models::user::User;
use crate::database::repositories::JobMetricRepository;

use crate::api::DbConn;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, post, routes, Route};
use rocket_db_pools::Connection;

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

/* ===================== ⚙️ JobMetric API Overview =====================

== 🛠️ CRUD ==
• POST   /metrics                             → Create new metric (NewJobMetric) → 201 Created (JobMetric)
• GET    /metrics/:id                         → Fetch metric by ID             → 200 OK (JobMetric)
• DELETE /metrics/:id                         → Delete metric by ID            → 204 No Content

== 🔍 Lookup & Search ==
• GET    /metrics/by_job/:job_id              → Metrics by Job ID              → 200 OK (Vec<JobMetric>)
• GET    /metrics/by_worker/:worker_id        → Metrics by Worker ID           → 200 OK (Vec<JobMetric>)
• GET    /metrics/recent/:job_id              → Most recent metric for Job     → 200 OK (Option<JobMetric>)
• GET    /metrics/chronological/:job_id       → Chronological metrics for Job  → 200 OK (Vec<JobMetric>)
• GET    /metrics/worker_stream/:worker_id    → Worker metric stream           → 200 OK (Vec<JobMetric>)

======================================================================== */

// ===== CRUD =======
#[post("/metrics", format = "json", data = "<new_metric>")]
pub async fn create_metric(
    mut db: Connection<DbConn>,
    new_metric: Json<NewJobMetric>,
    _user: User,
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
    _user: User,
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
    _user: User,
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
    _user: User,
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
    _user: User,
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
    _user: User,
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
    _user: User,
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
    _user: User,
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
