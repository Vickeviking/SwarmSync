use crate::enums::System::CoreEvent;

pub struct Harvester {}

impl Harvester {
    pub async fn init(mut core_event_rx: tokio::sync::broadcast::Receiver<CoreEvent>) {
        loop {
            match core_event_rx.recv().await {
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
