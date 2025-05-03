use common::database::models::user::User;
use common::database::models::worker::{NewWorkerStatus, WorkerStatus};
use common::database::repositories::WorkerStatusRepository;
use common::enums::workers::WorkerStatusEnum;
use common::rocket::DbConn;
use rocket::http::Status;
use rocket::response::status::{Custom, NoContent};
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, post, put, routes, Route};
use rocket_db_pools::Connection;

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
use chrono::Utc;
use serde::Deserialize;
// ===== CRUD =====
#[post("/worker-status", format = "json", data = "<new_status>")]
pub async fn create_worker_status(
    mut conn: Connection<DbConn>,
    mut new_status: Json<NewWorkerStatus>,
    _user: User,
) -> Result<Custom<Json<WorkerStatus>>, Custom<Json<Value>>> {
    if new_status.last_heartbeat.is_none() {
        new_status.last_heartbeat = Some(Utc::now().naive_utc());
    }
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
    _user: User,
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
    _user: User,
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
    _user: User,
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
    _user: User,
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

#[put("/worker-status/<id>/last-heartbeat")]
pub async fn update_last_heartbeat(
    mut conn: Connection<DbConn>,
    id: i32,
    _user: User,
) -> Result<Custom<Json<WorkerStatus>>, Custom<Json<Value>>> {
    WorkerStatusRepository::update_last_heartbeat(&mut conn, id)
        .await
        .map(|ws| Custom(Status::Ok, Json(ws)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

#[derive(Deserialize)]
pub struct UpdateActiveJobId {
    active_job_id: Option<i32>,
}

#[put("/worker-status/<id>/active-job-id", format = "json", data = "<data>")]
pub async fn update_active_job_id(
    mut conn: Connection<DbConn>,
    id: i32,
    data: Json<UpdateActiveJobId>,
    _user: User,
) -> Result<Custom<Json<WorkerStatus>>, Custom<Json<serde_json::Value>>> {
    WorkerStatusRepository::update_active_job_id(&mut conn, id, data.active_job_id)
        .await
        .map(|ws| Custom(Status::Ok, Json(ws)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({ "error": e.to_string() })),
            )
        })
}

#[put("/worker-status/<id>/uptime", format = "json", data = "<data>")]
pub async fn update_uptime(
    mut conn: Connection<DbConn>,
    id: i32,
    data: Json<serde_json::Value>,
    _user: User,
) -> Result<Custom<Json<WorkerStatus>>, Custom<Json<serde_json::Value>>> {
    let uptime = data.get("uptime").and_then(Value::as_i64).map(|v| v as i32);

    WorkerStatusRepository::update_uptime(&mut conn, id, uptime)
        .await
        .map(|ws| Custom(Status::Ok, Json(ws)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({ "error": e.to_string() })),
            )
        })
}

#[put("/worker-status/<id>/load-avg", format = "json", data = "<data>")]
pub async fn update_load_avg(
    mut conn: Connection<DbConn>,
    id: i32,
    data: Json<serde_json::Value>,
    _user: User,
) -> Result<Custom<Json<WorkerStatus>>, Custom<Json<serde_json::Value>>> {
    let load_avg = data.get("load_avg").and_then(Value::as_array).map(|arr| {
        arr.iter()
            .filter_map(Value::as_f64)
            .map(|f| f as f32)
            .collect::<Vec<f32>>()
    });

    WorkerStatusRepository::update_load_avg(&mut conn, id, load_avg)
        .await
        .map(|ws| Custom(Status::Ok, Json(ws)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({ "error": e.to_string() })),
            )
        })
}

#[put("/worker-status/<id>/last-error", format = "json", data = "<data>")]
pub async fn update_last_error(
    mut conn: Connection<DbConn>,
    id: i32,
    data: Json<serde_json::Value>,
    _user: User,
) -> Result<Custom<Json<WorkerStatus>>, Custom<Json<serde_json::Value>>> {
    let last_error = data
        .get("last_error")
        .and_then(Value::as_str)
        .map(String::from);

    WorkerStatusRepository::update_last_error(&mut conn, id, last_error)
        .await
        .map(|ws| Custom(Status::Ok, Json(ws)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({ "error": e.to_string() })),
            )
        })
}
