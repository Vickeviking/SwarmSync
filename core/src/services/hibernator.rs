use crate::enums::system::CoreEvent;

pub struct Hibernator {}

impl Hibernator {
    pub async fn init(mut core_event_rx: tokio::sync::broadcast::Receiver<CoreEvent>) {
        loop {
            match core_event_rx.recv().await {
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
