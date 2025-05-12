use crate::rocket_api::server_error;
use common::auth::{authorize_user, Credentials};
use common::database::models::user::User;
use common::database::repositories::UserRepository;
use common::rocket::{CacheConn, DbConn};
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::{json, Json, Value};
use rocket::{routes, Route};
use rocket_db_pools::deadpool_redis::redis::AsyncCommands;
use rocket_db_pools::Connection;

pub fn routes() -> Vec<Route> {
    routes![login]
}

// Login route, used to get a session token and login a user account
#[rocket::post("/login", format = "json", data = "<credentials>")]
pub async fn login(
    mut db: Connection<DbConn>,
    mut cache: Connection<CacheConn>,
    credentials: Json<Credentials>,
) -> Result<Value, Custom<Value>> {
    let user: User = UserRepository::find_by_username(&mut db, &credentials.username)
        .await
        .map_err(|e| server_error(e.into()))?
        .ok_or_else(|| Custom(Status::NotFound, json!("User not found")))?;

    let session_id = authorize_user(&user, credentials.into_inner())
        .map_err(|_| Custom(Status::Unauthorized, json!("Wrong credentials")))?;

    cache
        .set_ex::<String, i32, ()>(format!("sessions/{}", session_id), user.id, 3 * 60 * 60)
        .await
        .map_err(|e| server_error(e.into()))?;

    Ok(json!({
        "token": session_id,
    }))
}
