use crate::shared::{enums::system::CoreEvent, SharedResources};
use std::sync::Arc;
use tokio::sync::broadcast;

#[allow(dead_code)]
pub struct Reciever {
    shared_resources: Arc<SharedResources>,
    core_event_rx: broadcast::Receiver<CoreEvent>,
}

impl Reciever {
    pub fn new(shared_resources: Arc<SharedResources>) -> Self {
        Reciever {
            shared_resources: Arc::clone(&shared_resources),
            core_event_rx: shared_resources
                .get_service_channels()
                .subscribe_to_core_event(),
        }
    }

    pub async fn init(mut self) {
        loop {
            match self.core_event_rx.recv().await {
                Ok(CoreEvent::Startup) => println!("Reciever: Startup event received."),
                Ok(CoreEvent::Restart) => println!("Reciever: Restart event received."),
                Ok(CoreEvent::Shutdown) => {
                    println!("Reciever: Shutdown event received. Stopping...");
                    break;
                }
                Err(_) => {
                    println!("Reciever: Channel closed. Exiting...");
                    break;
                }
            }
        }
    }
}
