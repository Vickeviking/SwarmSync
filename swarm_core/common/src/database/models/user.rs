use crate::database::repositories::user::UserRepository;
use crate::database::schema::users;
use crate::rocket::{CacheConn, DbConn};
use chrono::NaiveDateTime;
use diesel::{prelude::*, Insertable, Queryable};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use rocket_db_pools::deadpool_redis::redis::AsyncCommands;
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};

/// User model, the user owning the seassion, connected to an account
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    // a hash of the password, salted
    pub password_hash: String,
    pub created_at: NaiveDateTime,
}

// Authguard for the user, when provided as argument in route, user is authenticated with token in
// request header
#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // Authorization: Bearer SESSION_ID_128_CHARACTERS_LONG
        let session_header = req
            .headers()
            .get_one("Authorization")
            .map(|v| v.split_whitespace().collect::<Vec<_>>())
            .filter(|v| v.len() == 2 && v[0] == "Bearer");
        // if session header is present
        if let Some(header_value) = session_header {
            // get redis conn
            let mut cache = req
                .guard::<Connection<CacheConn>>()
                .await
                .expect("Can not connect to Redis in request guard");
            // get db conn
            let mut db = req
                .guard::<Connection<DbConn>>()
                .await
                .expect("Can not connect to Postgres in request guard");
            // get cached user
            let result = cache
                .get::<String, i32>(format!("sessions/{}", header_value[1]))
                .await;
            // if cached user is found
            if let Ok(user_id) = result {
                // get user
                if let Ok(user) = UserRepository::find_by_id(&mut db, user_id).await {
                    return Outcome::Success(user);
                }
            }
        }

        Outcome::Error((Status::Unauthorized, ()))
    }
}

/// Insertable user
#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

/// Updateable user, model for updating a user through JSON,
/// Needed since we want optional password when updating
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateUserRequest {
    pub username: String,
    pub email: String,
    pub password: Option<String>,
}

/// New user request, used with JSON
/// Needed since we dont want derived traits to clash
#[derive(Debug, Deserialize, Serialize)]
pub struct NewUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// User response returned by routes,
/// Needed since password is not returned
#[derive(Serialize, Deserialize, Debug)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: NaiveDateTime,
}

/// get UserResponse from User
impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        UserResponse {
            id: u.id,
            username: u.username,
            email: u.email,
            created_at: u.created_at,
        }
    }
}
