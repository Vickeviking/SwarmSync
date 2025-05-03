use crate::utils::parsing;
use common::database::models::user::User;
use common::database::models::worker::{NewWorker, Worker};
use common::database::repositories::WorkerRepository;
use common::rocket::DbConn;

use rocket::http::Status;
use rocket::response::status::{Custom, NoContent};
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, patch, post, put, routes, Route};
use rocket_db_pools::Connection;

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
        update_worker,
    ]
}

/* ===================== ⚙️ Worker API Overview =====================

== 🛠️ CRUD ==
• POST    /workers                           → Create new worker (NewWorker)      → 201 Created (Worker)
• GET     /workers/:id                       → Fetch worker by ID                 → 200 OK (Worker)
• DELETE  /workers/:id                       → Delete worker by ID                → 204 No Content
• PUT     /workers/:id                       → Update worker by ID                → 200 OK (Worker)

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
    _user: User,
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

#[patch("/workers/<id>", format = "json", data = "<worker>")]
pub async fn update_worker(
    mut db: Connection<DbConn>,
    id: i32,
    worker: Json<Worker>,
    _user: User,
) -> Result<Json<Worker>, Custom<Json<serde_json::Value>>> {
    WorkerRepository::update(&mut db, id, worker.into_inner())
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({ "error": e.to_string() })),
            )
        })
}

#[get("/workers/<id>")]
pub async fn get_worker_by_id(
    mut conn: Connection<DbConn>,
    id: i32,
    _user: User,
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
    _user: User,
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
    _user: User,
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
    _user: User,
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
    _user: User,
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
    _user: User,
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
#[put("/workers/<id>/last-seen", format = "json", data = "<last_seen>")]
pub async fn update_last_seen(
    mut conn: Connection<DbConn>,
    id: i32,
    last_seen: Json<Value>,
    _user: User,
) -> Result<Custom<Json<Worker>>, Custom<Json<Value>>> {
    if let Some(last_seen_str) = last_seen.get("last_seen_at").and_then(Value::as_str) {
        // Use the parse_naive_datetime function for flexible date parsing
        let parsed = parsing::parse_naive_datetime(last_seen_str)
            .map_err(|e| Custom(Status::BadRequest, Json(json!({ "error": e }))))?;

        WorkerRepository::update_last_seen_at(&mut conn, id, parsed)
            .await
            .map(|w| Custom(Status::Ok, Json(w)))
            .map_err(|e| {
                Custom(
                    Status::InternalServerError,
                    Json(json!({ "error": e.to_string() })),
                )
            })
    } else {
        Err(Custom(
            Status::BadRequest,
            Json(json!({ "error": "Missing or invalid 'last_seen_at'" })),
        ))
    }
}
