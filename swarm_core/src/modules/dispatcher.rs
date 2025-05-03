use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use diesel_async::AsyncPgConnection;
use tokio::net::UdpSocket;
use tokio::sync::{broadcast::Receiver, RwLock};

use crate::commands::load_db_connection;
use crate::database::models::worker::Worker;
use crate::database::repositories::{WorkerRepository, WorkerStatusRepository};
use crate::shared::enums::log::{LogActionEnum, LogLevelEnum};
use crate::shared::enums::system::{CoreEvent, SystemModuleEnum};
use crate::shared::enums::workers::WorkerStatusEnum;
use crate::shared::SharedResources;
use crate::utils::Logger;
use anyhow::{Context, Result};

/// Shared state for dispatcher tasks
struct DispatcherState {
    worker_map: RwLock<HashMap<i32, Worker>>,
    status_map: RwLock<HashMap<i32, WorkerStatusEnum>>,
    last_seen: RwLock<HashMap<i32, Instant>>,
}

impl DispatcherState {
    /// Create new empty state
    fn new() -> Self {
        DispatcherState {
            worker_map: RwLock::new(HashMap::new()),
            status_map: RwLock::new(HashMap::new()),
            last_seen: RwLock::new(HashMap::new()),
        }
    }
}

pub struct Dispatcher {
    shared_resources: Arc<SharedResources>,
    core_event_rx: Receiver<CoreEvent>,
    state: Arc<DispatcherState>,
}

impl Dispatcher {
    pub fn new(shared_resources: Arc<SharedResources>) -> Self {
        Dispatcher {
            shared_resources: Arc::clone(&shared_resources),
            core_event_rx: shared_resources
                .get_service_channels()
                .subscribe_to_core_event(),
            state: Arc::new(DispatcherState::new()),
        }
    }

    pub async fn init(mut self) -> anyhow::Result<(), anyhow::Error> {
        loop {
            match self.core_event_rx.recv().await {
                Ok(CoreEvent::Startup) => {
                    // Load all workers into state (imagined function)
                    let mut conn: AsyncPgConnection = load_db_connection()
                        .await
                        .context("Failed to load DB connection")?;
                    let workers = WorkerRepository::list_all(&mut conn)
                        .await
                        .unwrap_or_default();
                    {
                        let mut map = self.state.worker_map.write().await;
                        let mut status = self.state.status_map.write().await;
                        let mut seen = self.state.last_seen.write().await;
                        for worker in workers {
                            let id = worker.id;
                            map.insert(id, worker.clone());
                            status.insert(id, WorkerStatusEnum::Offline);
                            seen.insert(id, Instant::now());
                        }
                    }
                    // Spawn UDP listener task
                    let udp_state = Arc::clone(&self.state);
                    let udp_resources = Arc::clone(&self.shared_resources);
                    tokio::spawn(async move {
                        let socket = UdpSocket::bind("0.0.0.0:5001").await.unwrap();
                        let mut buf = [0u8; 1024];
                        loop {
                            if let Ok((len, _addr)) = socket.recv_from(&mut buf).await {
                                if let Ok(text) = std::str::from_utf8(&buf[..len]) {
                                    let msg = text.trim().to_string();

                                    // *** debug‑rad ***
                                    //println!("DISPATCHER RX → {}", msg);

                                    let state = Arc::clone(&udp_state);
                                    let logger = udp_resources.get_logger();
                                    tokio::spawn(async move {
                                        handle_message(msg, state, logger).await;
                                    });
                                }
                            }
                        }
                    });

                    // Spawn unreachable sweep task (fast pulses)
                    let sweep_state = Arc::clone(&self.state);
                    let sweep_logger = self.shared_resources.get_logger();
                    let mut pulse_rx = self
                        .shared_resources
                        .get_pulse_subscriptions()
                        .subscribe_fast();

                    let mut conn: AsyncPgConnection = load_db_connection()
                        .await
                        .context("Failed to load DB connection")?;

                    tokio::spawn(async move {
                        // single DB connection moved in
                        loop {
                            if pulse_rx.recv().await.is_ok() {
                                let now = Instant::now();

                                // 1) grab both locks (write for status, read for last_seen)
                                let mut status_map = sweep_state.status_map.write().await;
                                let last_seen = sweep_state.last_seen.read().await;

                                // 2) collect all the worker IDs that need marking
                                let mut to_mark = Vec::new();
                                for (id, st) in status_map.iter() {
                                    // `id: &i32`, `st: &WorkerStatusEnum`
                                    if *st != WorkerStatusEnum::Offline {
                                        if let Some(last) = last_seen.get(id) {
                                            // `last: &Instant`
                                            if now.duration_since(*last) > Duration::from_secs(2) {
                                                to_mark.push(*id);
                                            }
                                        }
                                    }
                                }
                                // 3) now mutate and persist/log
                                for id in to_mark {
                                    status_map.insert(id, WorkerStatusEnum::Unreachable);
                                    if let Err(e) =
                                        WorkerStatusRepository::update_status_by_worker_id(
                                            &mut conn,
                                            id,
                                            WorkerStatusEnum::Unreachable,
                                        )
                                        .await
                                    {
                                        let err_msg =
                                            format!("DB error marking {} unreachable: {}", id, e);
                                        Logger::log(
                                            sweep_logger.clone(),
                                            LogLevelEnum::Error,
                                            SystemModuleEnum::Dispatcher,
                                            LogActionEnum::ClientConnected,
                                            None,
                                            None,
                                            None,
                                            Some(err_msg),
                                        )
                                        .await;
                                        continue;
                                    }
                                    let msg = format!("Worker {} marked UNREACHABLE", id);
                                    Logger::log(
                                        sweep_logger.clone(),
                                        LogLevelEnum::Warning,
                                        SystemModuleEnum::Dispatcher,
                                        LogActionEnum::ClientConnected,
                                        None,
                                        None,
                                        None,
                                        Some(msg),
                                    )
                                    .await;
                                }
                            } else {
                                break;
                            }
                        }
                    });
                }
                Ok(CoreEvent::Restart) => {
                    // (Optional) handle restart if needed
                }
                Ok(CoreEvent::Shutdown) => {
                    break;
                }
                Err(_) => {
                    break;
                }
            }
        }
        Ok(())
    }
}

/// Handle an incoming worker status message
async fn handle_message(
    msg: String,
    state: Arc<DispatcherState>,
    logger: Arc<Logger>,
) -> Result<()> {
    let parts: Vec<&str> = msg.split(',').collect();
    if parts.len() != 2 {
        return Ok(());
    }

    /* --- 1. Parse ---------------------------------------------------------------------------- */
    let id: i32 = match parts[0].trim().parse() {
        Ok(i) => i,
        Err(_) => return Ok(()),
    };
    let cmd = parts[1].trim();
    let new_status = match cmd {
        "CONNECT" | "IDLE" => WorkerStatusEnum::Idle,
        "BUSY" => WorkerStatusEnum::Busy,
        "DISCONNECT" => WorkerStatusEnum::Offline,
        _ => return Ok(()),
    };

    /* --- 2. Fast in‑memory update ------------------------------------------------------------- */
    {
        let mut status_map = state.status_map.write().await;
        let mut last_seen = state.last_seen.write().await;
        status_map.insert(id, new_status.clone());
        last_seen.insert(id, Instant::now());
    }

    /* --- 3. Persist to DB --------------------------------------------------------------------- */
    let mut conn: AsyncPgConnection = load_db_connection()
        .await
        .context("Failed DB connection in handle_message")?;

    let now_ts = Utc::now().naive_utc();

    // 3.1 update workers.last_seen_at (ignore error if row missing)
    let _ = WorkerRepository::update_last_seen_at(&mut conn, id, now_ts).await;

    // 3.2 update worker_status row (status + last_heartbeat)
    if let Ok(Some(ws)) = WorkerStatusRepository::find_by_worker_id(&mut conn, id).await {
        let _ = WorkerStatusRepository::update_status(&mut conn, ws.id, new_status.clone()).await;
        let _ = WorkerStatusRepository::update_last_heartbeat(&mut conn, ws.id).await;
    }

    /* --- 4. Log ------------------------------------------------------------------------------ */
    let log_msg = format!("Worker {id} status → {new_status:?}");
    Logger::log(
        logger,
        LogLevelEnum::Info,
        SystemModuleEnum::Dispatcher,
        LogActionEnum::ClientConnected,
        None,
        None,
        None,
        Some(log_msg),
    )
    .await;
    Ok(())
}
