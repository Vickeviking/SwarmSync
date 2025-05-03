use crate::core::shared_resources::SharedResources;
use crate::core::PulseBroadcaster;
use crate::core::{ModuleInitializer, ServiceInitializer};
use crate::modules::Logger;
use crate::services::{ServiceChannels, ServiceWiring};
use common::enums::system::CoreEvent;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

pub mod core;
pub mod modules;
pub mod rocket_api;
pub mod services;
pub mod utils;

fn main() {
    let runtime = Runtime::new().unwrap();
    let res = runtime.block_on(async { tokio_async_runtime().await });
    // Pretty print anyhow errors
    if let Err(err) = res {
        eprintln!("{:#}", err);
        std::process::exit(1);
    }
}

async fn tokio_async_runtime() -> anyhow::Result<(), anyhow::Error> {
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

    // ===== Create and start the services ====
    let initializer =
        ServiceInitializer::new(Arc::clone(&shared_resources), pulse_broadcaster).await;
    let shutdown_notify = initializer.shutdown_notify.clone();
    let _ = initializer.start()?;
    // ==== create and start modules ====
    let service_handles = ModuleInitializer::new(Arc::clone(&shared_resources));

    // Send startup signal to all services
    service_channels
        .send_event_to_all_services(CoreEvent::Startup)
        .await;
    println!("System started. Awaiting commands...");

    // Loop to handle shutdown events
    shutdown_notify.notified().await;
    println!("Notify‐based shutdown triggered.");
    service_channels
        .send_event_to_all_services(CoreEvent::Shutdown)
        .await;
    service_handles.join_tasks().await;
    println!("Shutdown complete.");

    Ok(())
}
