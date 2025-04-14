use crate::enums::system::CoreEvent::{Restart, Shutdown, Startup};
use crate::pulse_broadcaster::PulseBroadcaster;
use crate::service_channels::{ChannelType, EventPayload, ServiceChannels, ServiceWiring};
use crate::service_handles::ServiceHandles;
use services::logger::Logger;
use shared_resources::SharedResources;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::signal;
use tokio::sync::{mpsc, Mutex, Notify};

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
    // === Service channels =====
    let service_channels = Arc::new(ServiceChannels::new()); //subscribing one, only uses ref
    let service_wiring = Arc::new(Mutex::new(ServiceWiring::new()));
    // === Manually push in the channels ===

    /*      EXAMPLE of pushing in a mpsc channel between 2 modules
    let (corebridge_to_main_core_events_tx, corebridge_to_main_core_events_rx) =
        mpsc::unbounded_channel::<EventPayload>();
    {
        // Add one-one wirings here
        let service_wiring_lock = service_wiring.lock().await;

        // Add channels to the wiring structure
        service_wiring_lock
            .add_channel(
                ChannelType::CoreBridgeToMain_CoreEvents,
                corebridge_to_main_core_events_tx,
                corebridge_to_main_core_events_rx,
            )
            .await;
    } //drops lock
    */

    //broadcasting is seperated since broadcaster async loop needs owning access
    let pulse_broadcaster = PulseBroadcaster::new(service_channels.subscribe_to_core_event());
    let subscriptions = Arc::new(pulse_broadcaster.subscriptions());

    let shutdown_notify = Arc::new(Notify::new());

    //=== Shared resources ===
    let logger = Arc::new(Logger::new(
        service_channels.subscribe_to_core_event(),
        Arc::clone(&subscriptions),
    ));

    let shared_resources = Arc::new(SharedResources::new(
        logger,
        Arc::clone(&subscriptions),
        Arc::clone(&service_channels),
        Arc::clone(&service_wiring),
    ));

    // takes ownership of everything, but returns when done
    //use channels and spawn all services, recieves handles to all threads
    let service_handles = ServiceHandles::new(Arc::clone(&shared_resources));

    // ==== Send Startup signal to everyone =====
    service_channels.send_event_to_all_services(Startup).await;
    println!("System started. Awaiting commands...");

    // ==== Start Pulse BroadCaster ====
    tokio::spawn(async move {
        pulse_broadcaster.start().await;
    });

    // == Start ctrl-c watcher ===
    let shutdown_notify_clone = shutdown_notify.clone();
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl-c");
        println!("Ctrl-C detected. Triggering shutdown notify...");
        shutdown_notify_clone.notify_one(); // Wake up select!
    });

    // ==== Loop to handle incoming system manipulation events ====
    loop {
        tokio::select! {
            // Here, we intercept Ctrl-C (shutdown request)
            _ = shutdown_notify.notified() => {
                println!("Notify-based shutdown triggered (e.g. Ctrl-C).");
                service_channels.send_event_to_all_services(Shutdown).await;
                service_handles.join_tasks().await;
                println!("Shutdown complete.");
                break;
            }
        }
    }
}
