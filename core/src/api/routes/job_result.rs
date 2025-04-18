use crate::api::DbConn;
use crate::database::models::job::{JobResult, NewJobResult};
use crate::database::repositories::{
    JobAssignmentRepository, JobMetricRepository, JobRepository, JobResultRepository,
    LogEntryRepository, UserRepository, WorkerRepository, WorkerStatusRepository,
};
use crate::shared::{enums::system::CoreEvent, SharedResources};

use rocket::http::Status;
use rocket::response::status::{Custom, NoContent};
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, patch, post, routes, Build, Rocket, Route, Shutdown};
use rocket_db_pools::{Connection, Database};
use std::env;
use std::sync::Arc;
use tokio::sync::{broadcast::error::RecvError, mpsc, RwLock};

pub fn routes() -> Vec<Route> {
    routes![
        create_result,
        get_result,
        delete_result,
        get_results_by_job_id,
        list_results_for_job,
        get_most_recent_result_for_job,
        update_stdout,
        update_files
    ]
}

/* ===================== ⚙️ JobResult API Overview =====================

== 🛠️ CRUD ==
• POST    /results                   → Create new result (NewJobResult)         → 201 Created (JobResult)
• GET     /results/:id               → Fetch result by ID                       → 200 OK (JobResult)
• DELETE  /results/:id               → Delete result by ID                      → 204 No Content

== 🔍 Lookup & Search ==
• GET     /results/job/:job_id       → Results by Job ID                        → 200 OK (Vec<JobResult>)
• GET     /results/list/:job_id      → List results for Job                     → 200 OK (Vec<JobResult>)
• GET     /results/recent/:job_id    → Most recent result for Job               → 200 OK (Option<JobResult>)

== 🔄 Field Updates ==
• PATCH   /results/:id/stdout        → Update stdout field                      → 200 OK (JobResult)
• PATCH   /results/:id/files         → Update files field                       → 200 OK (JobResult)

======================================================================== */

// ===== CRUD =====
#[post("/results", format = "json", data = "<new_result>")]
pub async fn create_result(
    mut db: Connection<DbConn>,
    new_result: Json<NewJobResult>,
) -> Result<Custom<Json<JobResult>>, Custom<Json<serde_json::Value>>> {
    JobResultRepository::create(&mut db, new_result.into_inner())
        .await
        .map(|res| Custom(Status::Created, Json(res)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({ "error": e.to_string() })),
            )
        })
}

#[get("/results/<id>")]
pub async fn get_result(
    mut db: Connection<DbConn>,
    id: i32,
) -> Result<Json<JobResult>, Custom<Json<serde_json::Value>>> {
    JobResultRepository::find_by_id(&mut db, id)
        .await
        .map(Json)
        .map_err(|e| Custom(Status::NotFound, Json(json!({ "error": e.to_string() }))))
}

#[delete("/results/<id>")]
pub async fn delete_result(
    mut db: Connection<DbConn>,
    id: i32,
) -> Result<Status, Custom<Json<serde_json::Value>>> {
    JobResultRepository::delete(&mut db, id)
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
#[get("/results/job/<job_id>")]
pub async fn get_results_by_job_id(
    mut db: Connection<DbConn>,
    job_id: i32,
) -> Result<Json<Vec<JobResult>>, Custom<Json<serde_json::Value>>> {
    JobResultRepository::find_by_job_id(&mut db, job_id)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({ "error": e.to_string() })),
            )
        })
}

#[get("/results/list/<job_id>")]
pub async fn list_results_for_job(
    mut db: Connection<DbConn>,
    job_id: i32,
) -> Result<Json<Vec<JobResult>>, Custom<Json<serde_json::Value>>> {
    JobResultRepository::list_results_for_job(&mut db, job_id)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({ "error": e.to_string() })),
            )
        })
}

#[get("/results/recent/<job_id>")]
pub async fn get_most_recent_result_for_job(
    mut db: Connection<DbConn>,
    job_id: i32,
) -> Result<Json<Option<JobResult>>, Custom<Json<serde_json::Value>>> {
    JobResultRepository::get_most_recent_for_job(&mut db, job_id)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({ "error": e.to_string() })),
            )
        })
}

// ===== Field Updates =====
#[patch("/results/<id>/stdout", format = "json", data = "<new_stdout>")]
pub async fn update_stdout(
    mut db: Connection<DbConn>,
    id: i32,
    new_stdout: Json<Option<String>>,
) -> Result<Json<JobResult>, Custom<Json<serde_json::Value>>> {
    JobResultRepository::update_stdout(&mut db, id, new_stdout.into_inner())
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({ "error": e.to_string() })),
            )
        })
}

#[patch("/results/<id>/files", format = "json", data = "<new_files>")]
pub async fn update_files(
    mut db: Connection<DbConn>,
    id: i32,
    new_files: Json<Option<Vec<String>>>,
) -> Result<Json<JobResult>, Custom<Json<serde_json::Value>>> {
    JobResultRepository::update_files(&mut db, id, new_files.into_inner())
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({ "error": e.to_string() })),
            )
        })
}
