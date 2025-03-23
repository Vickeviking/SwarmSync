use crate::enums::system::{CoreEvent, Pulse};
use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::{broadcast::Receiver, mpsc::UnboundedSender, Mutex};
use tokio::time::{self, Duration};

pub struct PulseBroadcaster {
    pub slow_tx: broadcast::Sender<Pulse>,
    pub medium_tx: broadcast::Sender<Pulse>,
    pub fast_tx: broadcast::Sender<Pulse>,
    core_event_rx: Receiver<CoreEvent>,
}

impl PulseBroadcaster {
    pub fn new(core_event_rx: Receiver<CoreEvent>) -> Self {
        let (slow_tx, _) = broadcast::channel(100);
        let (medium_tx, _) = broadcast::channel(100);
        let (fast_tx, _) = broadcast::channel(100);

        Self {
            slow_tx,
            medium_tx,
            fast_tx,
            core_event_rx,
        }
    }

    pub fn subscribe_slow(&self) -> broadcast::Receiver<Pulse> {
        self.slow_tx.subscribe()
    }

    pub fn subscribe_medium(&self) -> broadcast::Receiver<Pulse> {
        self.medium_tx.subscribe()
    }

    pub fn subscribe_fast(&self) -> broadcast::Receiver<Pulse> {
        self.fast_tx.subscribe()
    }

    pub async fn start(mut self) {
        let mut slow_interval = time::interval(Duration::from_secs(10)); // 1 minute
        let mut medium_interval = time::interval(Duration::from_secs(1)); // 1 second
        let mut fast_interval = time::interval(Duration::from_millis(50)); // 50ms

        loop {
            select! {
                _ = slow_interval.tick() => {
                    let _ = self.slow_tx.send(Pulse::Slow);
                }
                _ = medium_interval.tick() => {
                    let _ = self.medium_tx.send(Pulse::Medium);
                }
                _ = fast_interval.tick() => {
                    let _ = self.fast_tx.send(Pulse::Fast);
                }
                event = self.core_event_rx.recv() => {
                    match event {
                        Ok(CoreEvent::Startup) => println!("Pulse broadcast: Startup event received."),
                        Ok(CoreEvent::Restart) => println!("Pulse broadcast: Restart event received."),
                        Ok(CoreEvent::Shutdown) => {
                            println!("Pulse broadcast: Shutdown event received. Stopping...");
                            break;
                        }
                        Err(_) => {
                            println!("broadcast: Channel closed. Exiting...");
                            break;
                        }
                    }
                }
            }
        }
    }
}
