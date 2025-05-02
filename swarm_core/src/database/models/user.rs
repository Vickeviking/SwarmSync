// ********** FILE CONTENT **********
//  Models for:
//      User
//
// ***********************************

use crate::database::schema::users;
use chrono::NaiveDateTime;
use diesel::{prelude::*, Insertable, Queryable};
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
