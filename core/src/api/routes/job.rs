use crate::api::DbConn;
use crate::database::models::job::{Job, NewJob};
use crate::database::repositories::{
    JobAssignmentRepository, JobMetricRepository, JobRepository, JobResultRepository,
    LogEntryRepository, UserRepository, WorkerRepository, WorkerStatusRepository,
};
use crate::shared::enums::job::JobStateEnum;
use crate::shared::{enums::system::CoreEvent, SharedResources};

use rocket::http::Status;
use rocket::response::status::{Custom, NoContent};
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, patch, post, routes, Build, Rocket, Route, Shutdown};

use chrono::{NaiveDateTime, Utc};
use rocket_db_pools::{Connection, Database};
use tokio::sync::{broadcast::error::RecvError, mpsc, RwLock};

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
======================== Routes Job ========================

== CRUD ==
• `POST /jobs`          → create(NewJob) -> Job
• `GET /jobs/:id`       → find_by_id(id) -> Job
• `PATCH /jobs/:id`     → update(id, Job) -> Job
• `DELETE /jobs/:id`    → delete(id) -> usize

== Lookup & Search ==
• `GET /jobs/search`    → search_by_job_name(user_id, query) -> Vec<Job>
• `GET /jobs/name/:str` → find_by_name(user_id, name) -> Option<Job>
• `GET /jobs/by_admin`  → list_by_admin(user_id, limit, offset) -> Vec<Job>
• `GET /jobs/state/:st` → list_by_state(state) -> Vec<Job>
• `GET /jobs/recent`    → get_recent_jobs(limit) -> Vec<Job>
• `GET /jobs/failed`    → get_failed_jobs(limit) -> Vec<Job>

== State Transitions ==
• `PATCH /jobs/:id/running`   → mark_running(id) -> Job
• `PATCH /jobs/:id/succeeded` → mark_succeeded(id) -> Job
• `PATCH /jobs/:id/failed`    → mark_failed(id, msg) -> Job

== Scheduling & Readiness ==
• `GET /jobs/scheduled`       → list_scheduled_jobs() -> Vec<Job>
• `GET /jobs/cron_due`        → list_due_cron_jobs(date, time) -> Vec<Job>
• `GET /jobs/ready`           → list_one_time_jobs_ready() -> Vec<Job>

== Aggregation & Stats ==
• `GET /jobs/stats/admins`    → get_job_counts_per_admin() -> Vec<(admin_id, job_count)>

== Assignment-related ==
• `GET /jobs/active/:worker`  → get_active_jobs_for_worker(worker_id) -> Vec<Job>
• `GET /jobs/assigned/:worker`→ find_jobs_assigned_to_worker(worker_id) -> Vec<Job>
• `GET /jobs/unassigned`      → list_jobs_with_no_assignment() -> Vec<Job>

*/

/** ========  Job model  ===========
#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(User))] // FK: user_id
pub struct Job {
    pub id: i32,
    pub user_id: i32,
    pub job_name: String,
    pub image_url: String,
    pub image_format: ImageFormatEnum,
    pub docker_flags: Option<Vec<Option<String>>>,
    pub output_type: OutputTypeEnum,
    pub output_paths: Option<Vec<Option<String>>>,
    pub schedule_type: ScheduleTypeEnum,
    pub cron_expression: Option<String>,
    pub notes: Option<String>,
    pub state: JobStateEnum,
    pub error_message: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = jobs)]
pub struct NewJob {
    pub user_id: i32,
    pub job_name: String,
    pub image_url: String,
    pub image_format: ImageFormatEnum,
    pub docker_flags: Option<Vec<Option<String>>>,
    pub output_type: OutputTypeEnum,
    pub output_paths: Option<Vec<Option<String>>>,
    pub schedule_type: ScheduleTypeEnum,
    pub cron_expression: Option<String>,
    pub notes: Option<String>,
}
*/

// ======= CRUD =======
#[post("/jobs", format = "json", data = "<new_job>")]
pub async fn create_job(
    mut db: Connection<DbConn>,
    new_job: Json<NewJob>,
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
) -> Result<Json<Vec<Job>>, Custom<Value>> {
    let current_time: NaiveDateTime =
        NaiveDateTime::parse_from_str(&date, &time).unwrap_or(Utc::now().naive_utc());
    JobRepository::list_due_cron_jobs(&mut db, current_time)
        .await
        .map(Json)
        .map_err(|e| Custom(Status::BadRequest, json!({ "error": e.to_string() })))
}

#[get("/jobs/ready")]
pub async fn list_ready_jobs(mut db: Connection<DbConn>) -> Result<Json<Vec<Job>>, Custom<Value>> {
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
