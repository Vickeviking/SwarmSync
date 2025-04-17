pub mod job;
pub mod job_assignment;
pub mod job_metric;
pub mod job_result;
pub mod log;
pub mod user;
pub mod worker;
pub mod worker_status;

use rocket::Route;

pub fn all_routes() -> Vec<Route> {
    [
        job::routes(),
        job_assignment::routes(),
        job_metric::routes(),
        job_result::routes(),
        log::routes(),
        user::routes(),
        worker::routes(),
        worker_status::routes(),
    ]
    .concat()
}
