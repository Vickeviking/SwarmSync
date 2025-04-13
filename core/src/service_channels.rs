use crate::enums::system::CoreEvent;
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc, RwLock};

/// Enum for different one-to-one channel types.
#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum ChannelType {
    CoreBridgeToMain_CoreEvents,
    CoreBridgeToMain_Notification, // not used just example showing that multiple can be added
}

// ====== Channel PAYLOAD ======
#[derive(Clone)]
pub enum EventPayload {
    CoreEvent(CoreEvent),
    NotificationEvent(NotificationEvent),
}

//WARNING: here is how to add payload

//Example on how to add payload
#[derive(Debug, Clone)]
pub struct NotificationEvent {
    pub notification: String,
}

impl NotificationEvent {
    pub fn new(notification: &str) -> Self {
        NotificationEvent {
            notification: notification.to_string(),
        }
    }
}

/// Global service channels for system-wide events.
/// - `core_event_tx` broadcasts events to all subscribers.
/// - `corebridge_to_main_tx` and `corebridge_to_main_rx` form a one-to-one channel.
pub struct ServiceChannels {
    pub core_event_tx: broadcast::Sender<CoreEvent>,
}

impl ServiceChannels {
    pub fn new() -> Self {
        let (core_event_tx, _) = broadcast::channel::<CoreEvent>(16);
        ServiceChannels { core_event_tx }
    }

    /// Broadcasts an event to all subscribers.
    pub async fn send_event_to_all_services(&self, event: CoreEvent) {
        let _ = self.core_event_tx.send(event);
    }

    /// Returns a new subscription to the global core event channel.
    pub fn subscribe_to_core_event(&self) -> broadcast::Receiver<CoreEvent> {
        self.core_event_tx.subscribe()
    }
}

impl Default for ServiceChannels {
    fn default() -> Self {
        Self::new()
    }
}

type ChannelPair = (
    Option<mpsc::UnboundedSender<EventPayload>>,
    Option<mpsc::UnboundedReceiver<EventPayload>>,
);

/// ServiceWiring encapsulates the one-to-one communication channels between modules.
/// This structure manages each channel identified by `ChannelType`.
pub struct ServiceWiring {
    inner: RwLock<HashMap<ChannelType, ChannelPair>>,
}

impl ServiceWiring {
    pub fn new() -> Self {
        ServiceWiring {
            inner: RwLock::new(HashMap::new()),
        }
    }

    /// Adds a new one-to-one channel identified by the given `ChannelType`.
    pub async fn add_channel(
        &self,
        channel: ChannelType,
        tx: mpsc::UnboundedSender<EventPayload>,
        rx: mpsc::UnboundedReceiver<EventPayload>,
    ) {
        let mut state = self.inner.write().await;
        state.insert(channel, (Some(tx), Some(rx)));
    }

    /// Takes ownership of the sender end for the specified channel,
    /// leaving the receiver end intact.
    pub async fn take_tx(
        &self,
        channel: ChannelType,
    ) -> Option<mpsc::UnboundedSender<EventPayload>> {
        let mut state = self.inner.write().await;
        state.get_mut(&channel)?.0.take()
    }

    /// Takes ownership of the receiver end for the specified channel,
    /// leaving the sender end intact.
    pub async fn take_rx(
        &self,
        channel: ChannelType,
    ) -> Option<mpsc::UnboundedReceiver<EventPayload>> {
        let mut state = self.inner.write().await;
        state.get_mut(&channel)?.1.take()
    }

    /// Gets a clone of the sender end for the specified channel.
    pub async fn get_tx(
        &self,
        channel: ChannelType,
    ) -> Option<mpsc::UnboundedSender<EventPayload>> {
        let state = self.inner.read().await;
        state.get(&channel).and_then(|(tx, _)| tx.clone())
    }

    /// Checks if both ends have been taken for cleanup (optional utility).
    pub async fn remove_if_empty(&self, channel: ChannelType) {
        let mut state = self.inner.write().await;
        if let Some((tx, rx)) = state.get(&channel) {
            if tx.is_none() && rx.is_none() {
                state.remove(&channel);
            }
        }
    }

    /// Checks if the sender end exists for the given channel.
    pub async fn tx_exists(&self, channel: ChannelType) -> bool {
        let state = self.inner.read().await;
        state.get(&channel).map_or(false, |(tx, _)| tx.is_some())
    }

    /// Checks if the receiver end exists for the given channel.
    pub async fn rx_exists(&self, channel: ChannelType) -> bool {
        let state = self.inner.read().await;
        state.get(&channel).map_or(false, |(_, rx)| rx.is_some())
    }
}

impl Default for ServiceWiring {
    fn default() -> Self {
        Self::new()
    }
}

#[tokio::test]
async fn test_service_wiring() {
    let wiring = ServiceWiring::new();
    let (tx, rx) = mpsc::unbounded_channel::<EventPayload>();
    let chan = ChannelType::CoreBridgeToMain_CoreEvents;

    // Add channel
    wiring.add_channel(chan.clone(), tx.clone(), rx).await;

    // Check tx and rx existence
    assert!(wiring.tx_exists(chan.clone()).await);
    assert!(wiring.rx_exists(chan.clone()).await);

    // Take tx only
    let taken_tx = wiring.take_tx(chan.clone()).await;
    assert!(taken_tx.is_some());
    assert!(!wiring.tx_exists(chan.clone()).await);
    assert!(wiring.rx_exists(chan.clone()).await);

    // Take rx next
    let taken_rx = wiring.take_rx(chan.clone()).await;
    assert!(taken_rx.is_some());
    assert!(!wiring.rx_exists(chan.clone()).await);

    // Now channel should be empty and removable
    wiring.remove_if_empty(chan.clone()).await;
    assert!(!wiring.tx_exists(chan.clone()).await);
    assert!(!wiring.rx_exists(chan.clone()).await);

    // Add a new channel and get cloned tx
    let (tx2, rx2) = mpsc::unbounded_channel::<EventPayload>();
    wiring.add_channel(chan.clone(), tx2.clone(), rx2).await;

    let cloned = wiring.get_tx(chan.clone()).await;
    assert!(cloned.is_some());
    let _ = cloned
        .unwrap()
        .send(EventPayload::NotificationEvent(NotificationEvent::new(
            "test",
        )));

    // Test overwriting channel
    let (tx3, rx3) = mpsc::unbounded_channel::<EventPayload>();
    wiring.add_channel(chan.clone(), tx3.clone(), rx3).await;

    let newer_tx = wiring.take_tx(chan.clone()).await;
    assert_eq!(newer_tx.unwrap().same_channel(&tx3), true);
}
