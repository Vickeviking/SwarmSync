use crate::{enums::system::CoreEvent, shared_resources::SharedResources};
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct Dispatcher {
    shared_resources: Arc<SharedResources>,
    core_event_rx: broadcast::Receiver<CoreEvent>,
}

impl Dispatcher {
    pub fn new(shared_resources: Arc<SharedResources>) -> Self {
        Dispatcher {
            shared_resources: Arc::clone(&shared_resources),
            core_event_rx: shared_resources
                .get_service_channels()
                .subscribe_to_core_event(),
        }
    }

    pub async fn init(mut self) {
        loop {
            match self.core_event_rx.recv().await {
                Ok(CoreEvent::Startup) => println!("Dispatcher: Startup event received."),
                Ok(CoreEvent::Restart) => println!("Dispatcher: Restart event received."),
                Ok(CoreEvent::Shutdown) => {
                    println!("Dispatcher: Shutdown event received. Stopping...");
                    break;
                }
                Err(_) => {
                    println!("Dispatcher: Channel closed. Exiting...");
                    break;
                }
            }
        }
    }
}
