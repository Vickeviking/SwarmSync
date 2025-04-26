## Internal Service Wiring and Message Passing

In a highly modular async system like SwarmSync’s Core, internal services (e.g., Scheduler, Dispatcher, Harvester, Logger, Hibernator) must be able to **communicate asynchronously**, **without tight coupling**, and with **ownership-safe isolation**. The orchestration layer accomplishes this via two internal systems:

### 1. **`ServiceChannels` — Lock-free Broadcast Subscriptions**

For *broadcast-style* communication (e.g., global events like system shutdowns, tick signals, or job status changes), we use `tokio::sync::broadcast`.

### Key Traits:

- Channel is **shared** across modules using `Arc<Mutex<ServiceChannels>>`.
- Every module can **subscribe independently** to the event stream.
- Events are sent **without owning** the receiver — perfect for services that may live dynamically or be restarted.
- Backed by:

  ```rust

  let (core_event_tx, _) = broadcast::channel::<CoreEvent>(16);
  ```

### Interface:

```rust

pub fn subscribe_to_core_event(&self) -> broadcast::Receiver<CoreEvent>
pub async fn send_event_to_all_services(&self, event: CoreEvent)
```

Each service calls `.subscribe_to_core_event()` on startup and then listens in its own task loop. No ownership conflicts, no shared mutability — and minimal boilerplate.

---

### 2. **`ServiceWiring` — One-to-One mpsc Wiring**

For *direct communication* between two specific modules, SwarmSync uses `tokio::mpsc::UnboundedSender`/`UnboundedReceiver` pairs, each labeled by a `ChannelType`.

This is encapsulated in a central wiring registry called `ServiceWiring`, which maps:

```rust

HashMap<ChannelType, (Option<Sender>, Option<Receiver
```

Each channel pair is registered once during init, and then each service may:

- **Take ownership** of one end (tx or rx) via `.take_tx()` or `.take_rx()` (only once)
- **Clone** the sender using `.get_tx()` (multi-producer allowed)
- **Inspect** whether a side has already been taken using `.tx_exists()` or `.rx_exists()`

This ensures **strict ownership**, while still allowing introspection for boot diagnostics and error reporting.

### Channel Structure:

```rust

pub enum EventPayload {
    DispatcherEvent(DispatcherEvent),
    SchedulerEvent(SchedulerEvent),
    // ...
}
pub enum ChannelType {
    SchedulerToDispatcher,
    DispatcherToHarvester,
    // ...
}
type ChannelPair = (Option<mpsc::UnboundedSender<EventPayload>>, Option<mpsc::UnboundedReceiver<EventPayload>>
```

### Example Usage Pattern:

1. On boot, the orchestrator calls `.add_channel(ChannelType::SchedulerToDispatcher, tx, rx)`
2. Scheduler takes its tx, Dispatcher takes its rx
3. During runtime, they can communicate via typed `EventPayload`s

---

### Shared Access: `shared_resource`

To reduce boilerplate and allow every service to access its assigned pipes, the `shared_resource` module wraps:

```rust

pub struct SharedResource {
    pub channels: Arc<Mutex<ServiceChannels>>,
    pub wiring: Arc<ServiceWiring>,
    // ...
}
```

Each service receives a cloned `Arc<SharedResource>` and can query or subscribe to the pipes it needs. For example:

```rust

let mut dispatcher_rx = shared.wiring.take_rx(ChannelType::SchedulerToDispatcher).await?;
let global_events = shared.channels.lock().await.subscribe_to_core_event();
```

This provides a consistent, async-safe, and ergonomic interface for all service interconnection needs.

---d

### Summary

| Pattern              | Tool                       | Use Case                          | Ownership Model |
| -------------------- | -------------------------- | --------------------------------- | --------------- |
| **Broadcast events** | `tokio::broadcast`         | Global signals (ticks, shutdowns) | Shared, no take |
| **Direct messages**  | `tokio::mpsc`              | Module-to-module event streams    | Take-only       |
| **Access pattern**   | `SharedResource` container | Pipe distribution post-init       | Arc + Mutex     |

---

This internal wiring system allows modules to be independently developed and tested, supports service restarts and hot-swaps, and removes tight dependencies from the runtime orchestration logic.
