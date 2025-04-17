use crate::shared::{enums::system::CoreEvent, SharedResources};
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct TaskArchive {
    shared_resources: Arc<SharedResources>,
    core_event_rx: broadcast::Receiver<CoreEvent>,
}

impl TaskArchive {
    pub fn new(shared_resources: Arc<SharedResources>) -> Self {
        TaskArchive {
            shared_resources: Arc::clone(&shared_resources),
            core_event_rx: shared_resources
                .get_service_channels()
                .subscribe_to_core_event(),
        }
    }

    pub async fn init(mut self) {
        loop {
            match self.core_event_rx.recv().await {
                Ok(CoreEvent::Startup) => println!("TaskArchive: Startup event received."),
                Ok(CoreEvent::Restart) => println!("TaskArchive: Restart event received."),
                Ok(CoreEvent::Shutdown) => {
                    println!("TaskArchive: Shutdown event received. Stopping...");
                    break;
                }
                Err(_) => {
                    println!("TaskArchive: Channel closed. Exiting...");
                    break;
                }
            }
        }
    }
}
