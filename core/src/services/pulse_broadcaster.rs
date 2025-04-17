use crate::enums::system::{CoreEvent, Pulse};
use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::time::{self, Duration};

/// Owns the broadcast channels and emits pulses over time.
/// Consumed and run inside one task only.
pub struct PulseBroadcaster {
    pub slow_tx: Sender<Pulse>,
    pub medium_tx: Sender<Pulse>,
    pub fast_tx: Sender<Pulse>,
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

    pub fn subscriptions(&self) -> PulseSubscriptions {
        PulseSubscriptions {
            slow_tx: self.slow_tx.clone(),
            medium_tx: self.medium_tx.clone(),
            fast_tx: self.fast_tx.clone(),
        }
    }

    /// Starts emitting pulses on separate intervals until shutdown.
    pub async fn start(mut self) {
        let mut slow_interval = time::interval(Duration::from_secs(10));
        let mut medium_interval = time::interval(Duration::from_secs(1));
        let mut fast_interval = time::interval(Duration::from_millis(50));

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
                            println!("Pulse broadcast: Channel closed. Exiting...");
                            break;
                        }
                    }
                }
            }
        }
    }
}

/// Lightweight read-only subscription clone of the broadcaster.
/// Can be shared freely across threads with no locking.
#[derive(Clone)]
pub struct PulseSubscriptions {
    pub slow_tx: Sender<Pulse>,
    pub medium_tx: Sender<Pulse>,
    pub fast_tx: Sender<Pulse>,
}

impl PulseSubscriptions {
    pub fn subscribe_slow(&self) -> Receiver<Pulse> {
        self.slow_tx.subscribe()
    }

    pub fn subscribe_medium(&self) -> Receiver<Pulse> {
        self.medium_tx.subscribe()
    }

    pub fn subscribe_fast(&self) -> Receiver<Pulse> {
        self.fast_tx.subscribe()
    }
}
