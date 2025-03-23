use crate::enums::system::CoreEvent;

pub struct Db {}

impl Db {
    pub async fn init(mut core_event_rx: tokio::sync::broadcast::Receiver<CoreEvent>) {
        loop {
            match core_event_rx.recv().await {
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
