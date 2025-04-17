pub mod enums;
pub mod models;
pub mod modules;
pub mod repositories;
pub mod schema;
pub mod services;
pub mod shared_resources;

use crate::enums::system::CoreEvent;
use crate::modules::{Logger, ModuleHandles};
use crate::services::{PulseBroadcaster, ServiceChannels, ServiceInitializer, ServiceWiring};

use crate::shared_resources::SharedResources;

use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

fn main() {
    let runtime = Runtime::new().unwrap();
    runtime.block_on(async {
        tokio_async_runtime().await;
    });
}

async fn tokio_async_runtime() {
    // Initialize necessary components
    let service_channels = Arc::new(ServiceChannels::new());
    let service_wiring = Arc::new(Mutex::new(ServiceWiring::new()));
    let pulse_broadcaster = PulseBroadcaster::new(service_channels.subscribe_to_core_event());
    let logger = Arc::new(Logger::new(
        service_channels.subscribe_to_core_event(),
        Arc::new(pulse_broadcaster.subscriptions()),
    ));
    let shared_resources = Arc::new(SharedResources::new(
        logger,
        Arc::new(pulse_broadcaster.subscriptions()),
        Arc::clone(&service_channels),
        service_wiring,
    ));

    // Create and start the services
    let initializer =
        ServiceInitializer::new(Arc::clone(&shared_resources), pulse_broadcaster).await;
    let shutdown_notify = initializer.shutdown_notify.clone();
    initializer.start();

    // create and start modules, constructor starts moduels
    let service_handles = ModuleHandles::new(Arc::clone(&shared_resources));

    // Send startup signal to all services
    service_channels
        .send_event_to_all_services(CoreEvent::Startup)
        .await;
    println!("System started. Awaiting commands...");

    // Loop to handle shutdown events
    loop {
        tokio::select! {
            _ = shutdown_notify.notified() => {
                println!("Notify-based shutdown triggered.");
                service_channels.send_event_to_all_services(CoreEvent::Shutdown).await;
                service_handles.join_tasks().await;
                println!("Shutdown complete.");
                break;
            }
        }
    }
}
