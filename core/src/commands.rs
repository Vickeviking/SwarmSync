use crate::api::auth;
use crate::database::models::user::{NewUser, User};
use crate::database::repositories::UserRepository;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use diesel_async::{AsyncConnection, AsyncPgConnection};

pub async fn load_db_connection() -> AsyncPgConnection {
    let database_url = std::env::var("DATABASE_URL").expect("Cannot load DB url from environment");
    AsyncPgConnection::establish(&database_url)
        .await
        .expect("Cannot connect to Postgres")
}

pub async fn create_user(username: String, email: String, password: String) {
    let mut c = load_db_connection().await;

    let password_hash = auth::hash_password(password).unwrap();
    let new_user = NewUser {
        username,
        email,
        password_hash,
    };

    let created = UserRepository::create(&mut c, new_user).await.unwrap();
    println!("✅ Created user: {} ({})", created.username, created.email);
}

pub async fn list_users(limit: i64, offset: i64) {
    let mut c = load_db_connection().await;

    let users = UserRepository::list_all(&mut c, limit, offset)
        .await
        .unwrap();
    println!("📄 Listing users (limit: {}, offset: {}):", limit, offset);
    for user in users {
        println!("({})- {} <{}>", user.id, user.username, user.email);
    }
}

pub async fn update_user(id: i32, username: String, email: String, password: String) {
    let mut c = load_db_connection().await;

    //time placeholder
    let d = NaiveDate::from_ymd_opt(2004, 1, 9).unwrap();
    let t = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    let dt = NaiveDateTime::new(d, t);

    let password_hash = auth::hash_password(password).unwrap();
    let user = User {
        id,
        username,
        email,
        password_hash,
        created_at: dt,
    };

    let updated = UserRepository::update(&mut c, id, user).await.unwrap();
    println!(
        "✏️ Updated user {} -> {} ({})",
        id, updated.username, updated.email
    );
}

pub async fn delete_user(id: i32) {
    let mut c = load_db_connection().await;

    let deleted = UserRepository::delete(&mut c, id).await.unwrap();
    if deleted > 0 {
        println!("🗑️ Deleted user with id {}", id);
    } else {
        println!("⚠️ No user found with id {}", id);
    }
}

pub async fn create_job(
    user_id: i32,
    job_name: String,
    docker_image: String,
    schedule: Option<String>,
) {
    // Logic to create a job for the user
    println!(
        "Creating job for user {}: {} with docker image {}",
        user_id, job_name, docker_image
    );
    if let Some(s) = schedule {
        println!("Scheduled at: {}", s);
    }
}

pub async fn remove_job(job_id: i32) {
    // Logic to remove a job
    println!("Removing job with ID: {}", job_id);
}

pub async fn list_jobs_by_user(user_id: i32) {
    // Logic to list jobs by user
    println!("Listing jobs for user ID: {}", user_id);
}
