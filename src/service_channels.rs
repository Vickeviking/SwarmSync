use crate::enums::System::CoreEvent;
use tokio::sync::{broadcast, mpsc};

pub struct ServiceChannels {
    pub core_event_tx: broadcast::Sender<CoreEvent>,
    pub core_event_subs: Vec<broadcast::Receiver<CoreEvent>>,
    pub core_event_manip_tx: mpsc::UnboundedSender<CoreEvent>,
    pub core_event_manip_rx: mpsc::UnboundedReceiver<CoreEvent>,

    // New: Service-to-service channels (for example, Hibernator -> Scheduler)
    pub hibernator_scheduler_tx: mpsc::Sender<CoreEvent>,
    pub scheduler_hibernator_rx: mpsc::Receiver<CoreEvent>,
}

impl ServiceChannels {
    pub fn new() -> Self {
        let (core_event_tx, _) = broadcast::channel::<CoreEvent>(16);

        // Create the core_event subscribers for different services
        let mut core_event_subs = Vec::new();
        for _ in 0..10 {
            core_event_subs.push(core_event_tx.subscribe());
        }

        // Manipulation channel
        let (core_event_manip_tx, core_event_manip_rx) = mpsc::unbounded_channel::<CoreEvent>();

        // New service-to-service channels (Example: Hibernator to Scheduler communication)
        let (hibernator_scheduler_tx, scheduler_hibernator_rx) = mpsc::channel(16);

        ServiceChannels {
            core_event_tx,
            core_event_subs,
            core_event_manip_tx,
            core_event_manip_rx,
            hibernator_scheduler_tx,
            scheduler_hibernator_rx,
        }
    }
    pub async fn send_event_to_all_services(&self, event: CoreEvent) {
        let _ = self.core_event_tx.send(event);
    }
}
