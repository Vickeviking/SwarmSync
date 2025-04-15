use crate::schema::admins;
use chrono::NaiveDateTime;
use diesel::{deserialize::FromSqlRow, prelude::*, Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = admins)]
pub struct Admin {
    pub id: i32,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = admins)]
pub struct NewAdmin {
    pub username: String,
    pub email: String,
    pub password_hash: String,
}
