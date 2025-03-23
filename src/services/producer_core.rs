use crate::enums::System::CoreEvent;

pub struct ProducerCore {}

impl ProducerCore {
    pub async fn init(mut core_event_rx: tokio::sync::broadcast::Receiver<CoreEvent>) {
        loop {
            match core_event_rx.recv().await {
                Ok(CoreEvent::Startup) => println!("ProducerCore: Startup event received."),
                Ok(CoreEvent::Restart) => println!("ProducerCore: Restart event received."),
                Ok(CoreEvent::Shutdown) => {
                    println!("ProducerCore: Shutdown event received. Stopping...");
                    break;
                }
                Err(_) => {
                    println!("ProducerCore: Channel closed. Exiting...");
                    break;
                }
            }
        }
    }
}
