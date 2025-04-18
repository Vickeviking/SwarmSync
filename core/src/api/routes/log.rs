use crate::api::DbConn;
use crate::database::models::log::{DBLogEntry, LogEntry, NewDBLogEntry};
use crate::database::repositories::{
    JobAssignmentRepository, JobMetricRepository, JobRepository, JobResultRepository,
    LogEntryRepository, UserRepository, WorkerRepository, WorkerStatusRepository,
};
use crate::shared::{enums::system::CoreEvent, enums::system::SystemModuleEnum, SharedResources};

use rocket::http::Status;
use rocket::response::status::{Custom, NoContent};
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, head, patch, post, put, routes, Build, Rocket, Route, Shutdown};
use rocket_db_pools::{Connection, Database};
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::{broadcast::error::RecvError, mpsc, RwLock};

pub fn routes() -> Vec<Route> {
    routes![
        create_log_entry,
        get_log_entry_by_id,
        delete_log_entry,
        update_log_entry,
        search_by_level,
        find_logs_by_module,
        search_by_action,
        list_all_logs,
        exists_log_by_level,
        exists_log_by_action,
    ]
}

/* ===================== ⚙️ Logs API Overview =====================

== 🛠️ CRUD ==
• POST    /logs                           → Create new log entry (NewDBLogEntry)   → 201 Created (LogEntry)
• GET     /logs/:id                       → Fetch log entry by ID                   → 200 OK (LogEntry)
• DELETE  /logs/:id                       → Delete log entry by ID                  → 204 No Content
• PUT     /logs/:id                       → Update log entry by ID                  → 200 OK (LogEntry)

== 🔍 Lookup & Search ==
• GET     /logs/search/level?q=:level     → Search logs by level                    → 200 OK (Vec<LogEntry>)
• GET     /logs/search/module?q=:module   → Search logs by module                   → 200 OK (Vec<LogEntry>)
• GET     /logs/search/action?q=:action   → Search logs by action                   → 200 OK (Vec<LogEntry>)
• GET     /logs?page=:page&limit=:limit   → List all logs (paginated)               → 200 OK (Vec<LogEntry>)

== 🔄 Field Updates ==
• PATCH   /logs/:id/msg                   → Update custom message                   → 200 OK (LogEntry)
• PATCH   /logs/:id/ttl                   → Update time-to-live                     → 200 OK (LogEntry)

== ⚡ Existence Checks ==
• HEAD    /logs/exists?action=:action     → Exists logs by action                   → 200 OK / 404 Not Found
• HEAD    /logs/exists?level=:level       → Exists logs by level                    → 200 OK / 404 Not Found

======================================================================== */

// ===== CRUD ======
#[post("/logs", format = "json", data = "<entry>")]
pub async fn create_log_entry(
    mut conn: Connection<DbConn>,
    entry: Json<NewDBLogEntry>,
) -> Result<Custom<Json<DBLogEntry>>, Custom<Json<serde_json::Value>>> {
    LogEntryRepository::create(&mut conn, entry.into_inner())
        .await
        .map(|log| Custom(Status::Created, Json(log)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

#[get("/logs/<id>")]
pub async fn get_log_entry_by_id(
    mut conn: Connection<DbConn>,
    id: i32,
) -> Result<Json<DBLogEntry>, Custom<Json<serde_json::Value>>> {
    LogEntryRepository::find_by_id(&mut conn, id)
        .await
        .map(Json)
        .map_err(|e| Custom(Status::NotFound, Json(json!({"error": e.to_string()}))))
}

#[delete("/logs/<id>")]
pub async fn delete_log_entry(
    mut conn: Connection<DbConn>,
    id: i32,
) -> Result<NoContent, Custom<Value>> {
    LogEntryRepository::delete(&mut conn, id)
        .await
        .map(|_| NoContent)
        .map_err(|e| Custom(Status::InternalServerError, json!({"error": e.to_string()})))
}

#[put("/logs/<id>", format = "json", data = "<entry>")]
pub async fn update_log_entry(
    mut conn: Connection<DbConn>,
    id: i32,
    entry: Json<DBLogEntry>,
) -> Result<Custom<Json<DBLogEntry>>, Custom<Json<serde_json::Value>>> {
    LogEntryRepository::update(&mut conn, id, entry.into_inner())
        .await
        .map(|log| Custom(Status::Created, Json(log)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

// ===== Lookup & search ======

#[get("/logs/search/level?<q>")]
pub async fn search_by_level(
    mut conn: Connection<DbConn>,
    q: String,
) -> Result<Custom<Json<Vec<DBLogEntry>>>, Custom<Json<serde_json::Value>>> {
    LogEntryRepository::search_by_level(&mut conn, &q)
        .await
        .map(|data| Custom(Status::Ok, Json(data)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({ "error": e.to_string() })),
            )
        })
}

#[get("/logs/search/module?<q>")]
pub async fn find_logs_by_module(
    mut conn: Connection<DbConn>,
    q: String,
) -> Result<Custom<Json<Vec<DBLogEntry>>>, Custom<Json<serde_json::Value>>> {
    let parsed = SystemModuleEnum::from_str(&q).map_err(|_| {
        Custom(
            Status::BadRequest,
            Json(json!({ "error": format!("Invalid module: '{}'", q) })),
        )
    })?;

    LogEntryRepository::find_logs_by_module(&mut conn, parsed)
        .await
        .map(|data| Custom(Status::Ok, Json(data)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

#[get("/logs/search/action?<q>")]
pub async fn search_by_action(
    mut conn: Connection<DbConn>,
    q: String,
) -> Result<Custom<Json<Vec<DBLogEntry>>>, Custom<Json<serde_json::Value>>> {
    LogEntryRepository::search_by_action(&mut conn, &q)
        .await
        .map(|data| Custom(Status::Ok, Json(data)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

#[get("/logs?<page>&<limit>")]
pub async fn list_all_logs(
    mut conn: Connection<DbConn>,
    page: Option<u32>,
    limit: Option<u32>,
) -> Result<Json<Vec<DBLogEntry>>, Custom<Json<serde_json::Value>>> {
    let limit = limit.unwrap_or(50);
    let offset = page.unwrap_or(0) * limit;

    LogEntryRepository::list_all(&mut conn, limit as i64, offset as i64)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({ "error": e.to_string() })),
            )
        })
}

// Exists

#[head("/logs/exists/level/<level>")]
pub async fn exists_log_by_level(
    mut conn: Connection<DbConn>,
    level: &str,
) -> Result<NoContent, Status> {
    match LogEntryRepository::exists_by_level(&mut conn, level).await {
        Ok(true) => Ok(NoContent),
        Ok(false) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[head("/logs/exists/action/<action>")]
pub async fn exists_log_by_action(
    mut conn: Connection<DbConn>,
    action: &str,
) -> Result<NoContent, Status> {
    match LogEntryRepository::exists_by_action(&mut conn, action).await {
        Ok(true) => Ok(NoContent),
        Ok(false) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}
