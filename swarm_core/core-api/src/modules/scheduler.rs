use crate::core::shared_resources::SharedResources;
use common::enums::system::CoreEvent;
use std::sync::Arc;
use tokio::sync::broadcast;

#[allow(dead_code)]
pub struct Scheduler {
    shared_resources: Arc<SharedResources>,
    core_event_rx: broadcast::Receiver<CoreEvent>,
}

impl Scheduler {
    pub fn new(shared_resources: Arc<SharedResources>) -> Self {
        Scheduler {
            shared_resources: Arc::clone(&shared_resources),
            core_event_rx: shared_resources
                .get_service_channels()
                .subscribe_to_core_event(),
        }
    }

    pub async fn init(mut self) {
        loop {
            match self.core_event_rx.recv().await {
                Ok(CoreEvent::Startup) => println!("Scheduler: Startup event received."),
                Ok(CoreEvent::Restart) => println!("Scheduler: Restart event received."),
                Ok(CoreEvent::Shutdown) => {
                    println!("Scheduler: Shutdown event received. Stopping...");
                    break;
                }
                Err(_) => {
                    println!("Scheduler: Channel closed. Exiting...");
                    break;
                }
            }
        }
    }
}
