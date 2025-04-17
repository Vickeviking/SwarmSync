use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Request, Response};
use std::error::Error;

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::response::status::Custom;
use rocket::serde::json::{json, Value};
use rocket_db_pools::deadpool_redis::redis::AsyncCommands;
use rocket_db_pools::Connection;

pub mod rocket_server;
pub mod routes;

// Fairing for rocket

#[derive(rocket_db_pools::Database)]
#[database("postgres")]
pub struct DbConn(rocket_db_pools::diesel::PgPool);
