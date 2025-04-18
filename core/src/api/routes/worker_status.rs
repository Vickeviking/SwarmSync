use crate::api::DbConn;
use crate::database::models::worker::{NewWorkerStatus, WorkerStatus};
use crate::database::repositories::{
    JobAssignmentRepository, JobMetricRepository, JobRepository, JobResultRepository,
    LogEntryRepository, UserRepository, WorkerRepository, WorkerStatusRepository,
};
use crate::shared::{enums::system::CoreEvent, enums::workers::WorkerStatusEnum, SharedResources};

use chrono::NaiveDateTime;
use rocket::http::Status;
use rocket::response::status::{Custom, NoContent};
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, post, put, routes, Build, Rocket, Route, Shutdown};
use rocket_db_pools::{Connection, Database};
use std::env;
use std::sync::Arc;
use tokio::sync::{broadcast::error::RecvError, mpsc, RwLock};

// === Mount routes ===
pub fn routes() -> Vec<Route> {
    routes![
        create_worker_status,
        get_worker_status_by_id,
        delete_worker_status,
        get_status_by_worker_id,
        update_status,
        update_last_heartbeat,
        update_active_job_id,
        update_uptime,
        update_load_avg,
        update_last_error,
    ]
}

/* ===================== ⚙️ WorkerStatus API Overview =====================

== 🛠️ CRUD ==
• POST    /worker-status                      → Create new status (NewWorkerStatus)     → 201 Created (WorkerStatus)
• GET     /worker-status/:id                  → Fetch status by ID                       → 200 OK (WorkerStatus)
• DELETE  /worker-status/:id                  → Delete status by ID                      → 204 No Content

== 🔍 Lookup & Search ==
• GET     /worker-status/worker/:worker_id    → Find status by Worker ID                → 200 OK (Option<WorkerStatus>)

== 🔄 State Updates ==
• PUT     /worker-status/:id/status           → Update overall status                   → 200 OK (WorkerStatus)
• PUT     /worker-status/:id/last-heartbeat  → Update last heartbeat timestamp         → 200 OK (WorkerStatus)
• PUT     /worker-status/:id/active-job-id    → Update active job ID                    → 200 OK (WorkerStatus)
• PUT     /worker-status/:id/uptime           → Update uptime in seconds                → 200 OK (WorkerStatus)
• PUT     /worker-status/:id/load-avg         → Update load average                     → 200 OK (WorkerStatus)
• PUT     /worker-status/:id/last-error       → Update last error message               → 200 OK (WorkerStatus)

======================================================================== */


// ===== CRUD =====
#[post("/worker-status", format = "json", data = "<new_status>")]
pub async fn create_worker_status(
    mut conn: Connection<DbConn>,
    new_status: Json<NewWorkerStatus>,
) -> Result<Custom<Json<WorkerStatus>>, Custom<Json<Value>>> {
    WorkerStatusRepository::create(&mut conn, new_status.into_inner())
        .await
        .map(|ws| Custom(Status::Created, Json(ws)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

#[get("/worker-status/<id>")]
pub async fn get_worker_status_by_id(
    mut conn: Connection<DbConn>,
    id: i32,
) -> Result<Json<WorkerStatus>, Custom<Json<Value>>> {
    WorkerStatusRepository::find_by_id(&mut conn, id)
        .await
        .map(Json)
        .map_err(|e| Custom(Status::NotFound, Json(json!({"error": e.to_string()}))))
}

#[delete("/worker-status/<id>")]
pub async fn delete_worker_status(
    mut conn: Connection<DbConn>,
    id: i32,
) -> Result<NoContent, Custom<Json<Value>>> {
    WorkerStatusRepository::delete_worker_status(&mut conn, id)
        .await
        .map(|_| NoContent)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

// ===== Lookup =====
#[get("/worker-status/worker/<worker_id>")]
pub async fn get_status_by_worker_id(
    mut conn: Connection<DbConn>,
    worker_id: i32,
) -> Result<Json<Option<WorkerStatus>>, Custom<Json<Value>>> {
    WorkerStatusRepository::find_by_worker_id(&mut conn, worker_id)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

// ===== State Updates =====
#[put("/worker-status/<id>/status", format = "json", data = "<status>")]
pub async fn update_status(
    mut conn: Connection<DbConn>,
    id: i32,
    status: Json<WorkerStatusEnum>,
) -> Result<Custom<Json<WorkerStatus>>, Custom<Json<Value>>> {
    WorkerStatusRepository::update_status(&mut conn, id, status.into_inner())
        .await
        .map(|ws| Custom(Status::Ok, Json(ws)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

#[put("/worker-status/<id>/last-heartbeat", format = "json", data = "<hb>")]
pub async fn update_last_heartbeat(
    mut conn: Connection<DbConn>,
    id: i32,
    hb: Json<NaiveDateTime>,
) -> Result<Custom<Json<WorkerStatus>>, Custom<Json<Value>>> {
    WorkerStatusRepository::update_last_heartbeat(&mut conn, id, hb.into_inner())
        .await
        .map(|ws| Custom(Status::Ok, Json(ws)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

#[put(
    "/worker-status/<id>/active-job-id",
    format = "json",
    data = "<job_id>"
)]
pub async fn update_active_job_id(
    mut conn: Connection<DbConn>,
    id: i32,
    job_id: Json<Option<i32>>,
) -> Result<Custom<Json<WorkerStatus>>, Custom<Json<Value>>> {
    WorkerStatusRepository::update_active_job_id(&mut conn, id, job_id.into_inner())
        .await
        .map(|ws| Custom(Status::Ok, Json(ws)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

#[put("/worker-status/<id>/uptime", format = "json", data = "<uptime>")]
pub async fn update_uptime(
    mut conn: Connection<DbConn>,
    id: i32,
    uptime: Json<Option<i32>>,
) -> Result<Custom<Json<WorkerStatus>>, Custom<Json<Value>>> {
    WorkerStatusRepository::update_uptime(&mut conn, id, uptime.into_inner())
        .await
        .map(|ws| Custom(Status::Ok, Json(ws)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

#[put("/worker-status/<id>/load-avg", format = "json", data = "<load_avg>")]
pub async fn update_load_avg(
    mut conn: Connection<DbConn>,
    id: i32,
    load_avg: Json<Option<Vec<f32>>>,
) -> Result<Custom<Json<WorkerStatus>>, Custom<Json<Value>>> {
    WorkerStatusRepository::update_load_avg(&mut conn, id, load_avg.into_inner())
        .await
        .map(|ws| Custom(Status::Ok, Json(ws)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

#[put(
    "/worker-status/<id>/last-error",
    format = "json",
    data = "<last_error>"
)]
pub async fn update_last_error(
    mut conn: Connection<DbConn>,
    id: i32,
    last_error: Json<Option<String>>,
) -> Result<Custom<Json<WorkerStatus>>, Custom<Json<Value>>> {
    WorkerStatusRepository::update_last_error(&mut conn, id, last_error.into_inner())
        .await
        .map(|ws| Custom(Status::Ok, Json(ws)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}
