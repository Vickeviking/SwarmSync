use crate::api::DbConn;
use crate::database::models::job::{Job, NewJob};
use crate::database::models::user::User;
use crate::database::repositories::JobRepository;

use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, patch, post, routes, Route};

use chrono::{NaiveDateTime, Utc};
use rocket_db_pools::Connection;

pub fn routes() -> Vec<Route> {
    routes![
        create_job,
        get_job,
        update_job,
        delete_job,
        search_jobs,
        find_job_by_name,
        list_jobs_by_admin,
        list_jobs_by_state,
        get_recent_jobs,
        get_failed_jobs,
        mark_job_running,
        mark_job_succeeded,
        mark_job_failed,
        list_scheduled_jobs,
        list_due_cron_jobs,
        list_ready_jobs,
        get_admin_job_counts,
        get_active_jobs_for_worker,
        get_jobs_assigned_to_worker,
        list_jobs_with_no_assignment,
    ]
}
/*
===================== 🚀 Job API Overview =====================

== 🛠️ CRUD ==
• POST   /jobs                     -> Creates a new job (NewJob)           → 201 Created (Job)
• GET    /jobs/:id                 -> Fetch job by ID                      → 200 OK (Job)
• PATCH  /jobs/:id                 -> Update job by ID (Job)               → 200 OK (Job)
• DELETE /jobs/:id                -> Delete job by ID                     → 204 No Content

== 🔍 Lookup & Search ==
• GET    /jobs/search?user_id&query         -> Fuzzy match jobs by name         → 200 OK (Vec<Job>)
• GET    /jobs/name/:user_id?name           -> Exact match job by name          → 200 OK (Vec<Job>)
• GET    /jobs/by_admin?user_id&limit&offset→ Jobs by a specific admin          → 200 OK (Vec<Job>)
• GET    /jobs/state/:state                 -> Jobs by job state enum           → 200 OK (Vec<Job>)
• GET    /jobs/recent?limit                 -> Most recent jobs (default 10)    → 200 OK (Vec<Job>)
• GET    /jobs/failed?limit                 -> Recently failed jobs             → 200 OK (Vec<Job>)

== 🔄 State Transitions ==
• PATCH  /jobs/:id/running        -> Mark job as running                   → 200 OK (Job)
• PATCH  /jobs/:id/succeeded      -> Mark job as succeeded                 → 200 OK (Job)
• PATCH  /jobs/:id/failed         -> Mark job as failed (with message)     → 200 OK (Job)

== ⏱️ Scheduling & Readiness ==
• GET    /jobs/scheduled                   -> All jobs with a schedule          → 200 OK (Vec<Job>)
• GET    /jobs/cron_due?date&time         -> Cron jobs due at a given time     → 200 OK (Vec<Job>)
• GET    /jobs/ready                       -> One-time jobs ready to run        → 200 OK (Vec<Job>)

== 📊 Aggregation & Stats ==
• GET    /jobs/stats/admins                -> Job count grouped by admin ID     → 200 OK (Vec<(i32, i64)>)

== 🤝 Assignment & Worker Routing ==
• GET    /jobs/active/:worker_id           -> Active jobs for worker            → 200 OK (Vec<Job>)
• GET    /jobs/assigned/:worker_id         -> Jobs assigned to worker           → 200 OK (Vec<Job>)
• GET    /jobs/unassigned                  -> Jobs with no worker assignment    → 200 OK (Vec<Job>)

===============================================================
*/

// ======= CRUD =======
#[post("/jobs", format = "json", data = "<new_job>")]
pub async fn create_job(
    mut db: Connection<DbConn>,
    new_job: Json<NewJob>,
    _user: User,
) -> Result<Custom<Json<Job>>, Custom<Json<serde_json::Value>>> {
    JobRepository::create(&mut db, new_job.into_inner())
        .await
        .map(|job| Custom(Status::Created, Json(job)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({ "error": e.to_string() })),
            )
        })
}

#[get("/jobs/<id>")]
pub async fn get_job(
    mut db: Connection<DbConn>,
    id: i32,
    _user: User,
) -> Result<Json<Job>, Custom<Json<serde_json::Value>>> {
    JobRepository::find_by_id(&mut db, id)
        .await
        .map(Json)
        .map_err(|e| Custom(Status::NotFound, Json(json!({ "error": e.to_string() }))))
}

#[patch("/jobs/<id>", format = "json", data = "<job>")]
pub async fn update_job(
    mut db: Connection<DbConn>,
    id: i32,
    job: Json<Job>,
    _user: User,
) -> Result<Json<Job>, Custom<Json<serde_json::Value>>> {
    JobRepository::update(&mut db, id, job.into_inner())
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({ "error": e.to_string() })),
            )
        })
}

#[delete("/jobs/<id>")]
pub async fn delete_job(
    mut db: Connection<DbConn>,
    id: i32,
    _user: User,
) -> Result<Status, Custom<Json<serde_json::Value>>> {
    JobRepository::delete(&mut db, id)
        .await
        .map(|_| Status::NoContent)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({ "error": e.to_string() })),
            )
        })
}

// ======== Lookup & Search ========
#[get("/jobs/search?<user_id>&<query>")]
pub async fn search_jobs(
    mut db: Connection<DbConn>,
    user_id: i32,
    query: &str,
    _user: User,
) -> Result<Json<Vec<Job>>, Custom<Value>> {
    JobRepository::search_by_job_name(&mut db, user_id, query)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                json!({ "error": e.to_string() }),
            )
        })
}

#[get("/jobs/name/<user_id>?<name>")]
pub async fn find_job_by_name(
    mut db: Connection<DbConn>,
    user_id: i32,
    name: &str,
    _user: User,
) -> Result<Json<Vec<Job>>, Custom<Value>> {
    JobRepository::search_by_job_name(&mut db, user_id, name)
        .await
        .map(Json)
        .map_err(|e| Custom(Status::NotFound, json!({ "error": e.to_string() })))
}

#[get("/jobs/by_admin?<user_id>&<limit>&<offset>")]
pub async fn list_jobs_by_admin(
    mut db: Connection<DbConn>,
    user_id: i32,
    limit: Option<i64>,
    offset: Option<i64>,
    _user: User,
) -> Result<Json<Vec<Job>>, Custom<Value>> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    JobRepository::list_by_admin(&mut db, user_id, limit, offset)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                json!({ "error": e.to_string() }),
            )
        })
}

#[get("/jobs/state/<state>")]
pub async fn list_jobs_by_state(
    mut db: Connection<DbConn>,
    state: String,
    _user: User,
) -> Result<Json<Vec<Job>>, Custom<Value>> {
    let enum_state = state
        .parse()
        .map_err(|_| Custom(Status::BadRequest, json!({ "error": "Invalid job state" })))?;
    JobRepository::list_by_state(&mut db, enum_state)
        .await
        .map(Json)
        .map_err(|e| Custom(Status::BadRequest, json!({ "error": e.to_string() })))
}

#[get("/jobs/recent?<limit>")]
pub async fn get_recent_jobs(
    mut db: Connection<DbConn>,
    limit: Option<i64>,
    _user: User,
) -> Result<Json<Vec<Job>>, Custom<Value>> {
    let limit = limit.unwrap_or(10);
    JobRepository::get_recent_jobs(&mut db, limit)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                json!({ "error": e.to_string() }),
            )
        })
}

#[get("/jobs/failed?<limit>")]
pub async fn get_failed_jobs(
    mut db: Connection<DbConn>,
    limit: Option<i64>,
    _user: User,
) -> Result<Json<Vec<Job>>, Custom<Value>> {
    let limit = limit.unwrap_or(10);
    JobRepository::get_failed_jobs(&mut db, limit)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                json!({ "error": e.to_string() }),
            )
        })
}

// ====== State Transition =======
#[patch("/jobs/<id>/running")]
pub async fn mark_job_running(
    mut db: Connection<DbConn>,
    id: i32,
    _user: User,
) -> Result<Json<Job>, Custom<Value>> {
    JobRepository::mark_running(&mut db, id)
        .await
        .map(Json)
        .map_err(|e| Custom(Status::Conflict, json!({ "error": e.to_string() })))
}

#[patch("/jobs/<id>/succeeded")]
pub async fn mark_job_succeeded(
    mut db: Connection<DbConn>,
    id: i32,
    _user: User,
) -> Result<Json<Job>, Custom<Value>> {
    JobRepository::mark_succeeded(&mut db, id)
        .await
        .map(Json)
        .map_err(|e| Custom(Status::Conflict, json!({ "error": e.to_string() })))
}

#[patch("/jobs/<id>/failed", format = "json", data = "<body>")]
pub async fn mark_job_failed(
    mut db: Connection<DbConn>,
    id: i32,
    body: Json<Value>,
    _user: User,
) -> Result<Json<Job>, Custom<Value>> {
    let message = body
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("Unspecified error")
        .to_string();

    JobRepository::mark_failed(&mut db, id, &message)
        .await
        .map(Json)
        .map_err(|e| Custom(Status::Conflict, json!({ "error": e.to_string() })))
}

// =========0 Schedulihng and readyness ==========

#[get("/jobs/scheduled")]
pub async fn list_scheduled_jobs(
    mut db: Connection<DbConn>,
    _user: User,
) -> Result<Json<Vec<Job>>, Custom<Value>> {
    JobRepository::list_scheduled_jobs(&mut db)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                json!({ "error": e.to_string() }),
            )
        })
}

#[get("/jobs/cron_due?<date>&<time>")]
pub async fn list_due_cron_jobs(
    mut db: Connection<DbConn>,
    date: String,
    time: String,
    _user: User,
) -> Result<Json<Vec<Job>>, Custom<Value>> {
    let current_time: NaiveDateTime =
        NaiveDateTime::parse_from_str(&date, &time).unwrap_or(Utc::now().naive_utc());
    JobRepository::list_due_cron_jobs(&mut db, current_time)
        .await
        .map(Json)
        .map_err(|e| Custom(Status::BadRequest, json!({ "error": e.to_string() })))
}

#[get("/jobs/ready")]
pub async fn list_ready_jobs(
    mut db: Connection<DbConn>,
    _user: User,
) -> Result<Json<Vec<Job>>, Custom<Value>> {
    JobRepository::list_one_time_jobs_ready(&mut db)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                json!({ "error": e.to_string() }),
            )
        })
}

// ======= Aggregation And Stats =========
#[get("/jobs/stats/admins")]
pub async fn get_admin_job_counts(
    mut db: Connection<DbConn>,
    _user: User,
) -> Result<Json<Vec<(i32, i64)>>, Custom<Value>> {
    JobRepository::get_job_counts_per_admin(&mut db)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                json!({ "error": e.to_string() }),
            )
        })
}

// ====== Assignment-related ===========
#[get("/jobs/active/<worker_id>")]
pub async fn get_active_jobs_for_worker(
    mut db: Connection<DbConn>,
    worker_id: i32,
    _user: User,
) -> Result<Json<Vec<Job>>, Custom<Value>> {
    JobRepository::get_active_jobs_for_worker(&mut db, worker_id)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                json!({ "error": e.to_string() }),
            )
        })
}

#[get("/jobs/assigned/<worker_id>")]
pub async fn get_jobs_assigned_to_worker(
    mut db: Connection<DbConn>,
    worker_id: i32,
    _user: User,
) -> Result<Json<Vec<Job>>, Custom<Value>> {
    JobRepository::find_jobs_assigned_to_worker(&mut db, worker_id)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                json!({ "error": e.to_string() }),
            )
        })
}

#[get("/jobs/unassigned")]
pub async fn list_jobs_with_no_assignment(
    mut db: Connection<DbConn>,
    _user: User,
) -> Result<Json<Vec<Job>>, Custom<Value>> {
    JobRepository::list_jobs_with_no_assignment(&mut db)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                json!({ "error": e.to_string() }),
            )
        })
}
