use crate::shared::{enums::system::CoreEvent, SharedResources};
use std::sync::Arc;
use tokio::sync::broadcast;

#[allow(dead_code)]
pub struct Hibernator {
    shared_resources: Arc<SharedResources>,
    core_event_rx: broadcast::Receiver<CoreEvent>,
}

impl Hibernator {
    pub fn new(shared_resources: Arc<SharedResources>) -> Self {
        Hibernator {
            shared_resources: Arc::clone(&shared_resources),
            core_event_rx: shared_resources
                .get_service_channels()
                .subscribe_to_core_event(),
        }
    }

    pub async fn init(mut self) {
        loop {
            match self.core_event_rx.recv().await {
                Ok(CoreEvent::Startup) => println!("Hibernator: Startup event received."),
                Ok(CoreEvent::Restart) => println!("Hibernator: Restart event received."),
                Ok(CoreEvent::Shutdown) => {
                    println!("Hibernator: Shutdown event received. Stopping...");
                    break;
                }
                Err(_) => {
                    println!("Hibernator: Channel closed. Exiting...");
                    break;
                }
            }
        }
    }
}
