use crate::api::routes;
use crate::api::{CacheConn, Cors, DbConn};
use crate::shared::{enums::system::CoreEvent, SharedResources};

use rocket::{routes, Build, Rocket};
use rocket_db_pools::Database;
use std::env;
use std::sync::Arc;
use tokio::sync::broadcast::error::RecvError;

#[allow(clippy::expect_used)]
pub async fn build_rocket(shared: Arc<SharedResources>) -> Rocket<Build> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url = env::var("REDIS_URL").expect("REDIS URL must be set");
    // Merge database URL into Rocket's config
    let figment = rocket::Config::figment()
        .merge(("address", "0.0.0.0"))
        .merge(("port", 8000))
        .merge(("databases.postgres.url", database_url.clone()))
        .merge(("databases.redis.url", redis_url.clone()));

    println!(
        "[Rocket] Loaded figment config with DB URL: {database_url} and Redis URL: {redis_url}"
    );

    rocket::custom(figment)
        .attach(Cors)
        .attach(CacheConn::init())
        .attach(DbConn::init())
        .manage(shared)
        .mount("/", routes::all_routes())
        .mount("/", routes![index])
}

#[rocket::get("/")]
fn index() -> &'static str {
    "SwarmSync is live."
}

#[allow(clippy::unwrap_used)]
pub async fn launch_rocket(shared: Arc<SharedResources>) {
    let mut shutdown_rx = shared.get_service_channels().subscribe_to_core_event();

    let rocket = build_rocket(Arc::clone(&shared)).await;

    let rocket = rocket.ignite().await.unwrap();
    // Get Rocket's shutdown handle before launching
    let shutdown_handle = rocket.shutdown();

    // Spawn Rocket in a task
    let rocket_handle = tokio::spawn(async move {
        if let Err(e) = rocket.launch().await {
            eprintln!("Rocket error: {:?}", e);
        }
    });

    // Listen for CoreEvent::Shutdown
    loop {
        match shutdown_rx.recv().await {
            Ok(CoreEvent::Shutdown) => {
                println!("Rocket: Shutdown signal received.");
                shutdown_handle.notify();
                break;
            }
            Ok(CoreEvent::Restart) => {
                println!("RocketServer: Restart event received.");
            }
            Ok(CoreEvent::Startup) => {
                println!("RocketServer: Startup event received.")
            }
            Err(RecvError::Closed) => {
                println!("Rocket: Event channel closed. Shutting down.");
                shutdown_handle.notify();
                break;
            }
            Err(_) => continue,
        }
    }

    let _ = rocket_handle.await;
}
