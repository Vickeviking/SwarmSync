use crate::enums::system::CoreEvent::{Restart, Shutdown, Startup};
use crate::pulse_broadcaster::PulseBroadcaster;
use crate::service_channels::ServiceChannels;
use crate::service_handles::ServiceHandles;
use services::logger::Logger;
use shared_resources::SharedResources;
use std::sync::Arc;
use tokio::runtime::Runtime;

pub mod db;
pub mod enums;
pub mod models;
pub mod pulse_broadcaster;
pub mod service_channels;
pub mod service_handles;
pub mod services;
pub mod shared_resources;

fn main() {
    let runtime = Runtime::new().unwrap();
    runtime.block_on(async {
        tokio_async_runtime().await;
    });
}

async fn tokio_async_runtime() {
    // All Internal Thread Communication
    let service_channels = ServiceChannels::new();
    let pulse_broadcaster = PulseBroadcaster::new(service_channels.subscribe_to_core_event());

    //=== Shared resources ===
    let logger = Arc::new(Logger::new(
        service_channels.subscribe_to_core_event(),
        pulse_broadcaster.subscribe_slow(),
    ));

    let shared_resources: SharedResources = SharedResources::new(logger);

    // takes ownership of everything, but returns when done
    //use channels and spawn all services, recieves handles to all threads
    let (service_handles, mut service_channels, pulse_broadcaster, shared_resources) =
        ServiceHandles::new(service_channels, pulse_broadcaster, shared_resources);

    // ==== Send Startup signal to everyone =====
    service_channels.send_event_to_all_services(Startup).await;
    println!("System started. Awaiting commands...");

    // ==== Start Pulse BroadCaster ====
    tokio::spawn(async move {
        pulse_broadcaster.start().await;
    });

    // ==== Loop to handle incoming system manipulation events ====
    while let Some(event) = service_channels.core_event_manip_rx.recv().await {
        match event {
            Shutdown => {
                println!("Shutdown command received. Notifying all tasks...");
                service_channels.send_event_to_all_services(Shutdown).await;
                service_handles.join_tasks().await;
                println!("All tasks have finished. System shutdown complete.");
                break; // Exit the loop to shut down main
            }
            Restart => {
                println!("Restart command received. Notifying all tasks...");
                service_channels.send_event_to_all_services(Restart).await;
            }
            Startup => {
                println!("Startup event received again. Ignoring...");
            }
        }
    }
}
