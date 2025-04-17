use crate::enums::system::CoreEvent;
use crate::shared_resources::SharedResources;
use rocket::figment::Figment;
use rocket::{Build, Rocket, Shutdown};
use std::sync::Arc;
use tokio::sync::{broadcast::error::RecvError, mpsc, RwLock};

pub async fn build_rocket(shared: Arc<SharedResources>) -> Rocket<Build> {
    rocket::build()
        .manage(shared)
        .mount("/", rocket::routes![index])
}

#[rocket::get("/")]
fn index() -> &'static str {
    "SwarmSync is live."
}

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
