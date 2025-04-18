use crate::api::DbConn;
use crate::database::models::worker::{NewWorker, Worker};
use crate::database::repositories::{
    JobAssignmentRepository, JobMetricRepository, JobRepository, JobResultRepository,
    LogEntryRepository, UserRepository, WorkerRepository, WorkerStatusRepository,
};

use crate::shared::{enums::system::CoreEvent, SharedResources};

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
        create_worker,
        get_worker_by_id,
        delete_worker,
        get_workers_by_admin,
        find_worker_by_label,
        find_worker_by_ip,
        list_workers_by_admin,
        update_last_seen,
    ]
}

/* ===================== ⚙️ Worker API Overview =====================

== 🛠️ CRUD ==
• POST    /workers                           → Create new worker (NewWorker)      → 201 Created (Worker)
• GET     /workers/:id                       → Fetch worker by ID                 → 200 OK (Worker)
• DELETE  /workers/:id                       → Delete worker by ID                → 204 No Content

== 🔍 Lookup & Search ==
• GET     /workers/admin/:admin_id           → Workers by Admin ID                → 200 OK (Vec<Worker>)
• GET     /workers/label/:label              → Find worker by label               → 200 OK (Option<Worker>)
• GET     /workers/ip/:ip_address            → Find worker by IP address          → 200 OK (Option<Worker>)
• GET     /workers/admin/:admin_id/list      → List workers by Admin (paginated)  → 200 OK (Vec<Worker>)

== 🔄 State Update ==
• PUT     /workers/:id/last-seen             → Update last-seen timestamp         → 200 OK (Worker)

======================================================================== */

// ===== CRUD =====
#[post("/workers", format = "json", data = "<new_worker>")]
pub async fn create_worker(
    mut conn: Connection<DbConn>,
    new_worker: Json<NewWorker>,
) -> Result<Custom<Json<Worker>>, Custom<Json<Value>>> {
    WorkerRepository::create(&mut conn, new_worker.into_inner())
        .await
        .map(|w| Custom(Status::Created, Json(w)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

#[get("/workers/<id>")]
pub async fn get_worker_by_id(
    mut conn: Connection<DbConn>,
    id: i32,
) -> Result<Json<Worker>, Custom<Json<Value>>> {
    WorkerRepository::find_by_id(&mut conn, id)
        .await
        .map(Json)
        .map_err(|e| Custom(Status::NotFound, Json(json!({"error": e.to_string()}))))
}

#[delete("/workers/<id>")]
pub async fn delete_worker(
    mut conn: Connection<DbConn>,
    id: i32,
) -> Result<NoContent, Custom<Json<Value>>> {
    WorkerRepository::delete_worker(&mut conn, id)
        .await
        .map(|_| NoContent)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

// ===== Lookup & Search =====
#[get("/workers/admin/<admin_id>")]
pub async fn get_workers_by_admin(
    mut conn: Connection<DbConn>,
    admin_id: i32,
) -> Result<Custom<Json<Vec<Worker>>>, Custom<Json<Value>>> {
    WorkerRepository::find_by_admin_id(&mut conn, admin_id)
        .await
        .map(|data| Custom(Status::Ok, Json(data)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

#[get("/workers/label/<label>")]
pub async fn find_worker_by_label(
    mut conn: Connection<DbConn>,
    label: String,
) -> Result<Json<Option<Worker>>, Custom<Json<Value>>> {
    WorkerRepository::find_by_label(&mut conn, &label)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

#[get("/workers/ip/<ip_address>")]
pub async fn find_worker_by_ip(
    mut conn: Connection<DbConn>,
    ip_address: String,
) -> Result<Json<Option<Worker>>, Custom<Json<Value>>> {
    WorkerRepository::find_by_ip_address(&mut conn, &ip_address)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

#[get("/workers/admin/<admin_id>/list?<page>&<limit>")]
pub async fn list_workers_by_admin(
    mut conn: Connection<DbConn>,
    admin_id: i32,
    page: Option<u32>,
    limit: Option<u32>,
) -> Result<Custom<Json<Vec<Worker>>>, Custom<Json<Value>>> {
    let limit = limit.unwrap_or(50);
    let offset = page.unwrap_or(0) as i64 * limit as i64;
    WorkerRepository::list_workers_by_admin(&mut conn, admin_id, limit as i64, offset)
        .await
        .map(|data| Custom(Status::Ok, Json(data)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

// ===== State Update =====
#[put("/workers/<id>/last-seen", format = "json", data = "<new_last_seen>")]
pub async fn update_last_seen(
    mut conn: Connection<DbConn>,
    id: i32,
    new_last_seen: Json<NaiveDateTime>,
) -> Result<Custom<Json<Worker>>, Custom<Json<Value>>> {
    WorkerRepository::update_last_seen_at(&mut conn, id, new_last_seen.into_inner())
        .await
        .map(|w| Custom(Status::Ok, Json(w)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}
