// ********** FILE CONTENT **********
//  Models for:
//      User
//
// ***********************************

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
        if let Some(header_value) = session_header {
            let mut cache = req
                .guard::<Connection<CacheConn>>()
                .await
                .expect("Can not connect to Redis in request guard");
            let mut db = req
                .guard::<Connection<DbConn>>()
                .await
                .expect("Can not connect to Postgres in request guard");

            let result = cache
                .get::<String, i32>(format!("sessions/{}", header_value[1]))
                .await;
            if let Ok(user_id) = result {
                if let Ok(user) = UserRepository::find_by_id(&mut db, user_id).await {
                    return Outcome::Success(user);
                }
            }
        }

        Outcome::Error((Status::Unauthorized, ()))
    }
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateUserRequest {
    pub username: String,
    pub email: String,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: NaiveDateTime,
}

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
