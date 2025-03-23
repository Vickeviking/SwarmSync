use crate::enums::system::CoreEvent;

pub struct TaskArchive {}

impl TaskArchive {
    pub async fn init(mut core_event_rx: tokio::sync::broadcast::Receiver<CoreEvent>) {
        loop {
            match core_event_rx.recv().await {
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
