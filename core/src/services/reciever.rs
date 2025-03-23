use crate::enums::system::CoreEvent;

pub struct Reciever {}

impl Reciever {
    pub async fn init(mut core_event_rx: tokio::sync::broadcast::Receiver<CoreEvent>) {
        loop {
            match core_event_rx.recv().await {
                Ok(CoreEvent::Startup) => println!("Reciever: Startup event received."),
                Ok(CoreEvent::Restart) => println!("Reciever: Restart event received."),
                Ok(CoreEvent::Shutdown) => {
                    println!("Reciever: Shutdown event received. Stopping...");
                    break;
                }
                Err(_) => {
                    println!("Reciever: Channel closed. Exiting...");
                    break;
                }
            }
        }
    }
}
