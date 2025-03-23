use crate::enums::System::CoreEvent;

pub struct Scheduler {}

impl Scheduler {
    pub async fn init(mut core_event_rx: tokio::sync::broadcast::Receiver<CoreEvent>) {
        loop {
            match core_event_rx.recv().await {
                Ok(CoreEvent::Startup) => println!("Scheduler: Startup event received."),
                Ok(CoreEvent::Restart) => println!("Scheduler: Restart event received."),
                Ok(CoreEvent::Shutdown) => {
                    println!("Scheduler: Shutdown event received. Stopping...");
                    break;
                }
                Err(_) => {
                    println!("Scheduler: Channel closed. Exiting...");
                    break;
                }
            }
        }
    }
}
