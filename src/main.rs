use crate::enums::System::CoreEvent::{Restart, Shutdown, Startup};
use crate::service_channels::ServiceChannels;
use crate::service_handles::ServiceHandles;
use tokio::runtime::Runtime;

pub mod db;
pub mod enums;
pub mod models;
pub mod service_channels;
pub mod service_handles;
pub mod services;

fn main() {
    let runtime = Runtime::new().unwrap();
    runtime.block_on(async {
        tokio_async_runtime().await;
    });
}

async fn tokio_async_runtime() {
    // All Internal Thread Communication
    let service_channels = ServiceChannels::new();
    //use channels and spawn all services, recieves handles to all threads
    let (service_handles, mut service_channels) = ServiceHandles::new(service_channels);

    // ==== Send Startup signal to everyone =====
    service_channels.send_event_to_all_services(Startup).await;
    println!("System started. Awaiting commands...");

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
