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

/* ======= WorkerStatus model ========
#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(Worker))]
#[diesel(table_name = worker_status)]
pub struct WorkerStatus {
    pub id: i32,
    pub worker_id: i32,
    pub status: WorkerStatusEnum,
    pub last_heartbeat: Option<NaiveDateTime>,
    pub active_job_id: Option<i32>,
    pub uptime_sec: Option<i32>,
    pub load_avg: Option<Vec<Option<f32>>>,
    pub last_error: Option<String>,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = worker_status)]
pub struct NewWorkerStatus {
    pub worker_id: i32,
    pub status: WorkerStatusEnum,
    pub last_heartbeat: Option<NaiveDateTime>,
    pub active_job_id: Option<i32>,
    pub uptime_sec: Option<i32>,
    pub load_avg: Option<Vec<Option<f32>>>,
    pub last_error: Option<String>,
}
*/

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
