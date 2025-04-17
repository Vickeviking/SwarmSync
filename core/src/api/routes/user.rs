use crate::database::repositories::{
    JobAssignmentRepository, JobMetricRepository, JobRepository, JobResultRepository,
    LogEntryRepository, UserRepository, WorkerRepository, WorkerStatusRepository,
};
use crate::shared::{enums::system::CoreEvent, SharedResources};

use rocket::{routes, Build, Rocket, Route, Shutdown};
use rocket_db_pools::Database;
use std::env;
use std::sync::Arc;
use tokio::sync::{broadcast::error::RecvError, mpsc, RwLock};

pub fn routes() -> Vec<Route> {
    routes![]
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
