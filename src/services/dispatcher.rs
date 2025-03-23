use crate::enums::System::CoreEvent;

pub struct Dispatcher {}

impl Dispatcher {
    pub async fn init(mut core_event_rx: tokio::sync::broadcast::Receiver<CoreEvent>) {
        loop {
            match core_event_rx.recv().await {
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
