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
/*======================== Routes Logs ========================

== CRUD ==
• `POST /logs`             → create(NewDBLogEntry) -> LogEntry
• `GET /logs/:id`          → find_by_id(id) -> LogEntry
• `DELETE /logs/:id`       → delete(id) -> usize
• `PUT /logs/:id`          → update(id, LogEntry) -> LogEntry

== Lookup & Search ==
• `GET /logs/search/level?q=INFO`       → search_by_level(query) -> Vec<LogEntry>
• `GET /logs/search/module?q=Scheduler` → find_logs_by_module(module: SystemModuleEnum)
• `GET /logs/search/action?q=dispatch`  → search_by_action(query)
• `GET /logs?page=x&limit=y`            → list_all(limit, offset)

== Field Updates ==
• `PATCH /logs/:id/msg`       → update_custom_msg(id, msg) -> LogEntry (not implemented yet, implied)
• `PATCH /logs/:id/ttl`       → update_expires_at(id, new_time) -> LogEntry (not implemented yet, implied)

== Existence Checks ==
• `HEAD /logs/exists?action=foo`  → exists_by_action(action) -> bool
• `HEAD /logs/exists?level=info`  → exists_by_level(level) -> bool
*/

/* ======= Log model =======
// Used in the application
pub struct LogEntry {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub level: LogLevelEnum,
    pub module: SystemModuleEnum,
    pub action: LogActionEnum,
    pub expires_at: NaiveDateTime,

    // Embed the payloads directly
    pub client_connected_payload: Option<ClientConnectedPayload>,
    pub job_submitted_payload: Option<JobSubmittedPayload>,
    pub job_completed_payload: Option<JobCompletedPayload>,
    pub custom_msg: Option<String>,
}

pub struct ClientConnectedPayload {
    pub ip: String,
    pub username: String,
}

pub struct JobSubmittedPayload {
    pub job_id: i32,
    pub from_module: SystemModuleEnum,
    pub to_module: SystemModuleEnum,
}

pub struct JobCompletedPayload {
    pub job_id: i32,
    pub success: bool,
}

// ====== DATABASE STORED STRUCTS ====

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = logs)]
pub struct DBLogEntry {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub level: LogLevelEnum,
    pub module: SystemModuleEnum,
    pub action: LogActionEnum,
    pub expires_at: NaiveDateTime,

    // Optional fields for the foreign keys to payload tables
    pub client_connected_ip: Option<String>, // Nullable IP field
    pub client_connected_username: Option<String>, // Nullable Username field

    pub job_submitted_job_id: Option<i32>, // Nullable Job ID
    pub job_submitted_from_module: Option<SystemModuleEnum>, // Nullable From module
    pub job_submitted_to_module: Option<SystemModuleEnum>, // Nullable To module

    pub job_completed_job_id: Option<i32>,   // Nullable Job ID
    pub job_completed_success: Option<bool>, // Nullable Success flag

    pub custom_msg: Option<String>, // Nullable custom message
}

#[derive(Debug, Insertable)]
#[diesel(table_name = logs)]
pub struct NewDBLogEntry {
    pub level: LogLevelEnum,
    pub module: SystemModuleEnum,
    pub action: LogActionEnum,
    pub expires_at: NaiveDateTime,

    // Optional fields for the foreign keys to payload tables
    pub client_connected_ip: Option<String>, // Nullable IP field
    pub client_connected_username: Option<String>, // Nullable Username field

    pub job_submitted_job_id: Option<i32>, // Nullable Job ID
    pub job_submitted_from_module: Option<SystemModuleEnum>, // Nullable From module
    pub job_submitted_to_module: Option<SystemModuleEnum>, // Nullable To module

    pub job_completed_job_id: Option<i32>,   // Nullable Job ID
    pub job_completed_success: Option<bool>, // Nullable Success flag

    pub custom_msg: Option<String>, // Nullable custom message
}
*/

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
