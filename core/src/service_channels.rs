use crate::enums::system::CoreEvent;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

/// Enum for different one-to-one channel types.
#[derive(Hash, Eq, PartialEq, Debug)]
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

/// ServiceWiring encapsulates the one-to-one communication channels between modules.
/// This structure manages each channel identified by `ChannelType`.
pub struct ServiceWiring {
    inner: RwLock<
        HashMap<
            ChannelType,
            (
                mpsc::UnboundedSender<EventPayload>,
                mpsc::UnboundedReceiver<EventPayload>,
            ),
        >,
    >,
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
        state.insert(channel, (tx, rx));
    }

    /// Takes ownership of the sender end for the specified channel,
    /// removing it from the wiring structure.
    pub async fn take_tx(
        &self,
        channel: ChannelType,
    ) -> Option<mpsc::UnboundedSender<EventPayload>> {
        let mut state = self.inner.write().await;
        state.remove(&channel).map(|(tx, _)| tx)
    }

    /// Takes ownership of the receiver end for the specified channel,
    /// removing it from the wiring structure.
    pub async fn take_rx(
        &self,
        channel: ChannelType,
    ) -> Option<mpsc::UnboundedReceiver<EventPayload>> {
        let mut state = self.inner.write().await;
        state.remove(&channel).map(|(_, rx)| rx)
    }

    /// Gets a clone of the sender end for the specified channel.
    pub async fn get_tx(
        &self,
        channel: ChannelType,
    ) -> Option<mpsc::UnboundedSender<EventPayload>> {
        let state = self.inner.read().await;
        state.get(&channel).map(|(tx, _)| tx.clone())
    }
}

#[tokio::test]
async fn test_service_wiring() {
    let wiring = ServiceWiring::new();
    let (tx, rx) = mpsc::unbounded_channel::<EventPayload>();

    wiring
        .add_channel(ChannelType::CoreBridgeToMain_CoreEvents, tx.clone(), rx)
        .await;

    let tx_taken = wiring
        .take_tx(ChannelType::CoreBridgeToMain_CoreEvents)
        .await;
    assert!(tx_taken.is_some());

    let (tx2, rx2) = mpsc::unbounded_channel::<EventPayload>();
    wiring
        .add_channel(ChannelType::CoreBridgeToMain_CoreEvents, tx2.clone(), rx2)
        .await;

    let tx_clone = wiring
        .get_tx(ChannelType::CoreBridgeToMain_CoreEvents)
        .await;
    assert!(tx_clone.is_some());
}
