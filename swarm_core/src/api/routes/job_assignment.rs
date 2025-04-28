use crate::api::DbConn;
use crate::database::models::job::{JobAssignment, NewJobAssignment};
use crate::database::repositories::JobAssignmentRepository;

use crate::database::models::user::User;
use crate::utils::parsing;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, patch, post, routes, Route};
use rocket_db_pools::Connection;

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

/* ===================== ⚙️ JobAssignment API Overview =====================

== 🛠️ CRUD ==
• POST   /assignments                    → Create new assignment (NewJobAssignment) → 201 Created (JobAssignment)
• GET    /assignments/:id               → Fetch assignment by ID → 200 OK (JobAssignment)
• DELETE /assignments/:id               → Delete assignment by ID → 204 No Content

== 🔍 Lookup & Search ==
• GET /assignments/by_job/:job_id                  → Assignments by Job ID → 200 OK (Vec<JobAssignment>)
• GET /assignments/by_worker/:worker_id            → Assignments by Worker ID → 200 OK (Vec<JobAssignment>)
• GET /assignments/lookup/:job_id/:worker_id       → Assignment by Job + Worker → 200 OK (Option<JobAssignment>)
• GET /assignments/by_worker/range?worker_id&start&end
                                                  → Assignments in time range for worker → 200 OK (Vec<JobAssignment>)
• GET /assignments/active                          → Currently active assignments → 200 OK (Vec<JobAssignment>)

== 🔄 State Transitions ==
• PATCH /assignments/:id/started   → Mark assignment as started (NaiveDateTime) → 200 OK (JobAssignment)
• PATCH /assignments/:id/finished  → Mark assignment as finished (NaiveDateTime) → 200 OK (JobAssignment)

======================================================================== */

// ========== CRUD =========
#[post("/assignments", format = "json", data = "<new_assignment>")]
async fn create_assignment(
    mut db: Connection<DbConn>,
    new_assignment: Json<NewJobAssignment>,
    _user: User,
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
    _user: User,
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
    _user: User,
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
    _user: User,
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
    _user: User,
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
    _user: User,
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
    _user: User,
) -> Result<Json<Vec<JobAssignment>>, Custom<Json<Value>>> {
    let start = parsing::parse_naive_datetime(start)
        .map_err(|_| Custom(Status::BadRequest, Json(json!({"error":"invalid start"}))))?;
    let end = parsing::parse_naive_datetime(end)
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
    _user: User,
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
    started_at: Json<Value>,
    _user: User,
) -> Result<Json<JobAssignment>, Custom<serde_json::Value>> {
    if let Some(started_at_str) = started_at.get("started_at").and_then(Value::as_str) {
        let started_at = parsing::parse_naive_datetime(started_at_str)
            .map_err(|e| Custom(Status::BadRequest, json!({ "error": e })))?;

        JobAssignmentRepository::update_started_at(&mut db, id, started_at)
            .await
            .map(Json)
            .map_err(|e| {
                Custom(
                    Status::InternalServerError,
                    json!({ "error": e.to_string() }),
                )
            })
    } else {
        Err(Custom(
            Status::BadRequest,
            json!({ "error": "Missing or invalid 'started_at' field" }),
        ))
    }
}

#[patch("/assignments/<id>/finished", format = "json", data = "<finished_at>")]
async fn update_finished_at(
    mut db: Connection<DbConn>,
    id: i32,
    finished_at: Json<Value>,
    _user: User,
) -> Result<Json<JobAssignment>, Custom<serde_json::Value>> {
    if let Some(finished_at_str) = finished_at.get("finished_at").and_then(Value::as_str) {
        let finished_at = parsing::parse_naive_datetime(finished_at_str)
            .map_err(|e| Custom(Status::BadRequest, json!({ "error": e })))?;

        JobAssignmentRepository::update_finished_at(&mut db, id, finished_at)
            .await
            .map(Json)
            .map_err(|e| {
                Custom(
                    Status::InternalServerError,
                    json!({ "error": e.to_string() }),
                )
            })
    } else {
        Err(Custom(
            Status::BadRequest,
            json!({ "error": "Missing or invalid 'finished_at' field" }),
        ))
    }
}
