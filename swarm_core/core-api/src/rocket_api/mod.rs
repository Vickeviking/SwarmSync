use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::{json, Value};
use rocket::{Request, Response};
use std::error::Error;

pub mod rocket_server;
pub mod routes;

// Fairing for rocket

#[rocket::options("/<_route_args..>")]
pub fn options(_route_args: Option<std::path::PathBuf>) {
    // Just to add CORS header via the fairing.
}

pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Append CORS headers in responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _req: &'r Request<'_>, res: &mut Response<'r>) {
        res.set_raw_header("Access-Control-Allow-Origin", "*");
        res.set_raw_header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE");
        res.set_raw_header("Access-Control-Allow-Headers", "*");
        res.set_raw_header("Access-Control-Allow-Credentials", "true");
    }
}

pub fn server_error(e: Box<dyn Error>) -> Custom<Value> {
    rocket::error!("{}", e);
    Custom(Status::InternalServerError, json!("Error"))
}
