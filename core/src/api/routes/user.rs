use crate::api::DbConn;
use crate::database::models::user::{NewUser, User};
use crate::database::repositories::{
    JobAssignmentRepository, JobMetricRepository, JobRepository, JobResultRepository,
    LogEntryRepository, UserRepository, WorkerRepository, WorkerStatusRepository,
};
use crate::shared::{enums::system::CoreEvent, SharedResources};

use rocket::http::Status;
use rocket::response::status::{Custom, NoContent};
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, head, post, put, routes, Build, Rocket, Route, Shutdown};
use rocket_db_pools::{Connection, Database};
use std::env;
use std::sync::Arc;
use tokio::sync::{broadcast::error::RecvError, mpsc, RwLock};

pub fn routes() -> Vec<Route> {
    routes![
        get_user_by_id,
        create_user,
        delete_user,
        update_user,
        find_user_by_email,
        find_user_by_username,
        search_by_username,
        search_by_email,
        list_users,
        exists_user_by_email,
        exists_user_by_username,
        users_with_jobs,
        user_job_counts,
    ]
}

/* ==================== Routes User ====================

== CRUD ==
• `GET /users/:id` → find_by_id(id) -> User
• `POST /users` → create(NewUser) -> User
• `DELETE /users/:id` → delete(id) -> usize
• `PUT /users/:id` → update(id, User) -> User

== Lookup ==
• `GET /users/email/:email` → find_by_email(email) -> Option<User>
• `GET /users/username/:username` → find_by_username(username) -> Option<User>

== Search ==
• `GET /users/search/username?q=alice` → search_by_username(query) -> Vec<User>
• `GET /users/search/email?q=example.com` → search_by_email(query) -> Vec<User>

== Listing ==
• `GET /users?page=x&limit=y` → list_all(limit, offset) -> Vec<User>

== Existence Checks ==
• `HEAD /users/exists/email/:email` → exists_by_email(email) -> bool
• `HEAD /users/exists/username/:username` → exists_by_username(username) -> bool

== Relational & Aggregation ==
• `GET /users/with-jobs` → find_users_with_jobs() -> Vec<User>
• `GET /users/job-counts` → get_user_with_job_counts() -> Vec<(User, i64)>

*/

/* ====== User model ========
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password_hash: String,
}
*/

// === CRUD ===
#[get("/users/<id>")]
pub async fn get_user_by_id(
    mut conn: Connection<DbConn>,
    id: i32,
) -> Result<Json<User>, Custom<Json<Value>>> {
    UserRepository::find_by_id(&mut conn, id)
        .await
        .map(Json)
        .map_err(|e| Custom(Status::NotFound, Json(json!({"error": e.to_string()}))))
}

#[post("/users", format = "json", data = "<new_user>")]
pub async fn create_user(
    mut conn: Connection<DbConn>,
    new_user: Json<NewUser>,
) -> Result<Custom<Json<User>>, Custom<Json<Value>>> {
    UserRepository::create(&mut conn, new_user.into_inner())
        .await
        .map(|u| Custom(Status::Created, Json(u)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

#[delete("/users/<id>")]
pub async fn delete_user(
    mut conn: Connection<DbConn>,
    id: i32,
) -> Result<NoContent, Custom<Json<Value>>> {
    UserRepository::delete(&mut conn, id)
        .await
        .map(|_| NoContent)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

#[put("/users/<id>", format = "json", data = "<user>")]
pub async fn update_user(
    mut conn: Connection<DbConn>,
    id: i32,
    user: Json<User>,
) -> Result<Custom<Json<User>>, Custom<Json<Value>>> {
    UserRepository::update(&mut conn, id, user.into_inner())
        .await
        .map(|u| Custom(Status::Ok, Json(u)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

// === Lookup ===
#[get("/users/email/<email>")]
pub async fn find_user_by_email(
    mut conn: Connection<DbConn>,
    email: String,
) -> Result<Json<Option<User>>, Custom<Json<Value>>> {
    UserRepository::find_by_email(&mut conn, &email)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

#[get("/users/username/<username>")]
pub async fn find_user_by_username(
    mut conn: Connection<DbConn>,
    username: String,
) -> Result<Json<Option<User>>, Custom<Json<Value>>> {
    UserRepository::find_by_username(&mut conn, &username)
        .await
        .map(Json)
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

// === Search ===
#[get("/users/search/username?<q>")]
pub async fn search_by_username(
    mut conn: Connection<DbConn>,
    q: String,
) -> Result<Custom<Json<Vec<User>>>, Custom<Json<Value>>> {
    UserRepository::search_by_username(&mut conn, &q)
        .await
        .map(|data| Custom(Status::Ok, Json(data)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

#[get("/users/search/email?<q>")]
pub async fn search_by_email(
    mut conn: Connection<DbConn>,
    q: String,
) -> Result<Custom<Json<Vec<User>>>, Custom<Json<Value>>> {
    UserRepository::search_by_email(&mut conn, &q)
        .await
        .map(|data| Custom(Status::Ok, Json(data)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

// === Listing ===
#[get("/users?<page>&<limit>")]
pub async fn list_users(
    mut conn: Connection<DbConn>,
    page: Option<u32>,
    limit: Option<u32>,
) -> Result<Custom<Json<Vec<User>>>, Custom<Json<Value>>> {
    let limit = limit.unwrap_or(50);
    let offset = page.unwrap_or(0) * limit;

    UserRepository::list_all(&mut conn, limit as i64, offset as i64)
        .await
        .map(|data| Custom(Status::Ok, Json(data)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

// === Existence Checks ===
#[head("/users/exists/email/<email>")]
pub async fn exists_user_by_email(
    mut conn: Connection<DbConn>,
    email: &str,
) -> Result<NoContent, Status> {
    match UserRepository::exists_by_email(&mut conn, email).await {
        Ok(true) => Ok(NoContent),
        Ok(false) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[head("/users/exists/username/<username>")]
pub async fn exists_user_by_username(
    mut conn: Connection<DbConn>,
    username: &str,
) -> Result<NoContent, Status> {
    match UserRepository::exists_by_username(&mut conn, username).await {
        Ok(true) => Ok(NoContent),
        Ok(false) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

// === Relational & Aggregation ===
#[get("/users/with-jobs")]
pub async fn users_with_jobs(
    mut conn: Connection<DbConn>,
) -> Result<Custom<Json<Vec<User>>>, Custom<Json<Value>>> {
    UserRepository::find_users_with_jobs(&mut conn)
        .await
        .map(|data| Custom(Status::Ok, Json(data)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}

#[get("/users/job-counts")]
pub async fn user_job_counts(
    mut conn: Connection<DbConn>,
) -> Result<Custom<Json<Vec<(User, i64)>>>, Custom<Json<Value>>> {
    UserRepository::get_user_with_job_counts(&mut conn)
        .await
        .map(|data| Custom(Status::Ok, Json(data)))
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                Json(json!({"error": e.to_string()})),
            )
        })
}
