use crate::services::{PulseBroadcaster, ServiceChannels};
use crate::shared_resources::SharedResources;

use std::sync::Arc;
use tokio::signal;
use tokio::sync::Notify;

pub struct ServiceInitializer {
    pub pulse_broadcaster: Option<PulseBroadcaster>,
    pub shutdown_notify: Arc<Notify>,
}

impl ServiceInitializer {
    pub async fn new(
        shared_resources: Arc<SharedResources>,
        pulse_broadcaster: PulseBroadcaster,
    ) -> Self {
        // Initialize the services
        let shutdown_notify = Arc::new(Notify::new());

        ServiceInitializer {
            pulse_broadcaster: Some(pulse_broadcaster),
            shutdown_notify,
        }
    }

    pub fn start(mut self) {
        // Start Pulse Broadcaster
        let pb = self
            .pulse_broadcaster
            .take()
            .expect("pulse broadcast did not exist in start() service_initializer.rs");
        tokio::spawn(async move {
            pb.start().await;
        });

        // Start Ctrl-C watcher
        let shutdown_notify_clone = self.shutdown_notify.clone();
        tokio::spawn(async move {
            signal::ctrl_c().await.expect("Failed to listen for ctrl-c");
            println!("Ctrl-C detected. Triggering shutdown notify...");
            shutdown_notify_clone.notify_one();
        });
    }
}
