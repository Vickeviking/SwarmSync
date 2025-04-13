use crate::enums::system::CoreEvent;
use crate::SharedResources;
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct Db {
    shared_resources: Arc<SharedResources>,
    core_event_rx: broadcast::Receiver<CoreEvent>,
}

impl Db {
    pub fn new(shared_resources: Arc<SharedResources>) -> Self {
        Db {
            shared_resources: Arc::clone(&shared_resources),
            core_event_rx: shared_resources
                .get_service_channels()
                .subscribe_to_core_event(),
        }
    }

    pub async fn init(mut self) {
        loop {
            match self.core_event_rx.recv().await {
                Ok(CoreEvent::Startup) => println!("Db: Startup event received."),
                Ok(CoreEvent::Restart) => println!("Db: Restart event received."),
                Ok(CoreEvent::Shutdown) => {
                    println!("Db: Shutdown event received. Stopping...");
                    break;
                }
                Err(_) => {
                    println!("Db: Channel closed. Exiting...");
                    break;
                }
            }
        }
    }
}
