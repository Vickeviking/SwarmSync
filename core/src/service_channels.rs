use crate::enums::system::CoreEvent;
use tokio::sync::{broadcast, mpsc};

pub struct ServiceChannels {
    pub core_event_tx: broadcast::Sender<CoreEvent>,
    pub core_event_manip_tx: mpsc::UnboundedSender<CoreEvent>,
    pub core_event_manip_rx: mpsc::UnboundedReceiver<CoreEvent>,
}

impl ServiceChannels {
    pub fn new() -> Self {
        let (core_event_tx, _) = broadcast::channel::<CoreEvent>(16);
        // Manipulation channel
        let (core_event_manip_tx, core_event_manip_rx) = mpsc::unbounded_channel::<CoreEvent>();

        ServiceChannels {
            core_event_tx,
            core_event_manip_tx,
            core_event_manip_rx,
        }
    }
    pub async fn send_event_to_all_services(&self, event: CoreEvent) {
        let _ = self.core_event_tx.send(event);
    }

    pub fn subscribe_to_core_event(&self) -> broadcast::Receiver<CoreEvent> {
        self.core_event_tx.subscribe()
    }
}

impl Default for ServiceChannels {
    fn default() -> Self {
        Self::new() // Use the new method to create a default instance
    }
}
