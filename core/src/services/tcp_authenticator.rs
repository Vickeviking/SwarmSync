use crate::{enums::system::CoreEvent, shared_resources::SharedResources};
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct TCPAuthenticator {
    shared_resources: Arc<SharedResources>,
    core_event_rx: broadcast::Receiver<CoreEvent>,
}

impl TCPAuthenticator {
    pub fn new(shared_resources: Arc<SharedResources>) -> Self {
        TCPAuthenticator {
            shared_resources: Arc::clone(&shared_resources),
            core_event_rx: shared_resources
                .get_service_channels()
                .subscribe_to_core_event(),
        }
    }

    pub async fn init(mut self) {
        loop {
            match self.core_event_rx.recv().await {
                Ok(CoreEvent::Startup) => println!("TCPAuthenticator: Startup event received."),
                Ok(CoreEvent::Restart) => println!("TCPAuthenticator: Restart event received."),
                Ok(CoreEvent::Shutdown) => {
                    println!("TCPAuthenticator: Shutdown event received. Stopping...");
                    break;
                }
                Err(_) => {
                    println!("TCPAuthenticator: Channel closed. Exiting...");
                    break;
                }
            }
        }
    }
}
