use crate::enums::System::CoreEvent;

pub struct Logger {}

impl Logger {
    pub async fn init(mut core_event_rx: tokio::sync::broadcast::Receiver<CoreEvent>) {
        loop {
            match core_event_rx.recv().await {
                Ok(CoreEvent::Startup) => println!("Logger: Startup event received."),
                Ok(CoreEvent::Restart) => println!("Logger: Restart event received."),
                Ok(CoreEvent::Shutdown) => {
                    println!("Logger: Shutdown event received. Stopping...");
                    break;
                }
                Err(_) => {
                    println!("Logger: Channel closed. Exiting...");
                    break;
                }
            }
        }
    }
}
