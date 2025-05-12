///! The harvester module
use crate::core::shared_resources::SharedResources;
use common::enums::system::CoreEvent;
use std::sync::Arc;
use tokio::sync::broadcast;

#[allow(dead_code)]
pub struct Harvester {
    shared_resources: Arc<SharedResources>,
    core_event_rx: broadcast::Receiver<CoreEvent>,
}

impl Harvester {
    pub fn new(shared_resources: Arc<SharedResources>) -> Self {
        Harvester {
            shared_resources: Arc::clone(&shared_resources),
            core_event_rx: shared_resources
                .get_service_channels()
                .subscribe_to_core_event(),
        }
    }

    pub async fn init(mut self) {
        loop {
            match self.core_event_rx.recv().await {
                Ok(CoreEvent::Startup) => println!("Harvester: Startup event received."),
                Ok(CoreEvent::Restart) => println!("Harvester: Restart event received."),
                Ok(CoreEvent::Shutdown) => {
                    println!("Harvester: Shutdown event received. Stopping...");
                    break;
                }
                Err(_) => {
                    println!("Harvester: Channel closed. Exiting...");
                    break;
                }
            }
        }
    }
}
