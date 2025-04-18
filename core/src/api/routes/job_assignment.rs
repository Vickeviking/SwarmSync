use crate::api::DbConn;
use crate::database::models::job::{JobAssignment, NewJobAssignment};
use crate::database::repositories::{
    JobAssignmentRepository, JobMetricRepository, JobRepository, JobResultRepository,
    LogEntryRepository, UserRepository, WorkerRepository, WorkerStatusRepository,
};
use crate::shared::enums::job::JobStateEnum;
use crate::shared::{enums::system::CoreEvent, SharedResources};

use chrono::NaiveDateTime;
use rocket::http::Status;
use rocket::response::status::{Custom, NoContent};
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, patch, post, put, routes, Build, Rocket, Route, Shutdown};
use rocket_db_pools::{Connection, Database};
use std::env;
use std::sync::Arc;
use tokio::sync::{broadcast::error::RecvError, mpsc, RwLock};

pub fn routes() -> Vec<Route> {
    routes![
        create_assignment,
        get_assignment_by_id,
        delete_assignment,
        get_assignments_by_job_id,
        get_assignments_by_worker_id,
        lookup_assignment,
        get_assignments_for_worker_in_range,
        get_active_assignments,
        update_started_at,
        update_finished_at
    ]
}

/* ================================== Routes JobAssignment ==================================

== CRUD ==
• `POST /assignments`        → create(NewJobAssignment) -> JobAssignment
• `GET /assignments/:id`     → find_by_id(id) -> JobAssignment
• `DELETE /assignments/:id`  → delete(id) -> usize

== Lookup & Search ==
• `GET /assignments/by_job/:job_id`             → find_by_job_id(job_id) -> Vec<JobAssignment>
• `GET /assignments/by_worker/:worker_id`       → find_by_worker_id(worker_id) -> Vec<JobAssignment>
• `GET /assignments/lookup/:job_id/:worker_id`  → find_assignment_by_job_and_worker(job_id, worker_id) -> Option<JobAssignment>
• `GET /assignments/by_worker/range`            → find_assignments_for_worker_in_time_range(worker_id, start, end) -> Vec<JobAssignment>
• `GET /assignments/active` → list_active_assignments() -> Vec<JobAssignment>

== State Updates ==
• `PATCH /assignments/:id/started`   → update_started_at(id, started_at) -> JobAssignment
• `PATCH /assignments/:id/finished`  → update_finished_at(id, finished_at) -> JobAssignment

*/

/* ========== JobAssignment model ============
#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(Job))] // FK: job_id
#[diesel(belongs_to(Worker))] // FK: worker_id
pub struct JobAssignment {
    pub id: i32,
    pub job_id: i32,
    pub worker_id: i32,
    pub assigned_at: NaiveDateTime,
    pub started_at: Option<NaiveDateTime>,
    pub finished_at: Option<NaiveDateTime>,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = job_assignments)]
pub struct NewJobAssignment {
    pub job_id: i32,
    pub worker_id: i32,
}
*/

// ========== CRUD =========
#[post("/assignments", format = "json", data = "<new_assignment>")]
async fn create_assignment(
    mut db: Connection<DbConn>,
    new_assignment: Json<NewJobAssignment>,
) -> Result<Custom<Json<JobAssignment>>, Custom<Json<serde_json::Value>>> {
    JobAssignmentRepository::create(&mut db, new_assignment.into_inner())
        .await
        .map(|job| Custom(Status::Created, Json(job)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({ "error": e.to_string() })),
            )
        })
}

#[get("/assignments/<id>")]
async fn get_assignment_by_id(
    mut db: Connection<DbConn>,
    id: i32,
) -> Result<Json<JobAssignment>, Custom<Json<serde_json::Value>>> {
    JobAssignmentRepository::find_by_id(&mut db, id)
        .await
        .map(Json)
        .map_err(|e| Custom(Status::NotFound, Json(json!({ "error": e.to_string() }))))
}

#[delete("/assignments/<id>")]
async fn delete_assignment(
    mut db: Connection<DbConn>,
    id: i32,
) -> Result<Status, Custom<Json<serde_json::Value>>> {
    JobAssignmentRepository::delete(&mut db, id)
        .await
        .map(|_| Status::NoContent)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string() })),
            )
        })
}

// ========== Lookup & Search =======

#[get("/assignments/by_job/<job_id>")]
async fn get_assignments_by_job_id(
    mut db: Connection<DbConn>,
    job_id: i32,
) -> Result<Json<Vec<JobAssignment>>, Custom<serde_json::Value>> {
    JobAssignmentRepository::find_by_job_id(&mut db, job_id)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                json!({ "error": e.to_string() }),
            )
        })
}

#[get("/assignments/by_worker/<worker_id>")]
async fn get_assignments_by_worker_id(
    mut db: Connection<DbConn>,
    worker_id: i32,
) -> Result<Json<Vec<JobAssignment>>, Custom<serde_json::Value>> {
    JobAssignmentRepository::find_by_worker_id(&mut db, worker_id)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                json!({ "error": e.to_string() }),
            )
        })
}

#[get("/assignments/lookup/<job_id>/<worker_id>")]
async fn lookup_assignment(
    mut db: Connection<DbConn>,
    job_id: i32,
    worker_id: i32,
) -> Result<Json<Option<JobAssignment>>, Custom<serde_json::Value>> {
    JobAssignmentRepository::find_assignment_by_job_and_worker(&mut db, job_id, worker_id)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                json!({ "error": e.to_string() }),
            )
        })
}

#[get("/assignments/by_worker/range?<worker_id>&<start>&<end>")]
async fn get_assignments_for_worker_in_range(
    mut db: Connection<DbConn>,
    worker_id: i32,
    start: &str,
    end: &str,
) -> Result<Json<Vec<JobAssignment>>, Custom<Json<Value>>> {
    let start = NaiveDateTime::parse_from_str(start, "%Y-%m-%d %H:%M:%S")
        .map_err(|_| Custom(Status::BadRequest, Json(json!({"error":"invalid start"}))))?;
    let end = NaiveDateTime::parse_from_str(end, "%Y-%m-%d %H:%M:%S")
        .map_err(|_| Custom(Status::BadRequest, Json(json!({"error":"invalid end"}))))?;

    JobAssignmentRepository::find_assignments_for_worker_in_time_range(
        &mut db, worker_id, start, end,
    )
    .await
    .map(Json)
    .map_err(|e| {
        Custom(
            Status::InternalServerError,
            Json(json!({ "error": e.to_string() })),
        )
    })
}

#[get("/assignments/active")]
async fn get_active_assignments(
    mut db: Connection<DbConn>,
) -> Result<Json<Vec<JobAssignment>>, Custom<serde_json::Value>> {
    JobAssignmentRepository::list_active_assignments(&mut db)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                json!({ "error": e.to_string() }),
            )
        })
}

// ========== Updates ==========

#[patch("/assignments/<id>/started", format = "json", data = "<started_at>")]
async fn update_started_at(
    mut db: Connection<DbConn>,
    id: i32,
    started_at: Json<NaiveDateTime>,
) -> Result<Json<JobAssignment>, Custom<serde_json::Value>> {
    JobAssignmentRepository::update_started_at(&mut db, id, started_at.into_inner())
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                json!({ "error": e.to_string() }),
            )
        })
}

#[patch("/assignments/<id>/finished", format = "json", data = "<finished_at>")]
async fn update_finished_at(
    mut db: Connection<DbConn>,
    id: i32,
    finished_at: Json<NaiveDateTime>,
) -> Result<Json<JobAssignment>, Custom<serde_json::Value>> {
    JobAssignmentRepository::update_finished_at(&mut db, id, finished_at.into_inner())
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                json!({ "error": e.to_string() }),
            )
        })
}
