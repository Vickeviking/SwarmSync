use crate::enums::system::CoreEvent;

pub struct TCPAuthenticator {}

impl TCPAuthenticator {
    pub async fn init(mut core_event_rx: tokio::sync::broadcast::Receiver<CoreEvent>) {
        loop {
            match core_event_rx.recv().await {
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
