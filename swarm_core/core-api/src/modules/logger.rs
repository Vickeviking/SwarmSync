// ================= logger.rs ==================
use crate::core::PulseSubscriptions;
use chrono::{NaiveDateTime, Utc};
use common::commands::load_db_connection;
use common::database::models::log::{
    ClientConnectedPayload, JobCompletedPayload, JobSubmittedPayload, LogEntry, NewDBLogEntry,
};
use common::database::repositories::LogEntryRepository;
use common::enums::log::{LogActionEnum, LogLevelEnum};
use common::enums::system::{CoreEvent, Pulse, SystemModuleEnum};
use diesel::prelude::{ExpressionMethods, QueryDsl};
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl; // async execute/delete
use std::sync::Arc;
use tokio::select;
use tokio::sync::{broadcast::Receiver, Mutex, RwLock};

/// Central logger accessed by all modules
pub struct Logger {
    buffer_logs: RwLock<Vec<LogEntry>>, // flushed on pulse/shutdown
    core_event_rx: Mutex<Receiver<CoreEvent>>, // lifecycle events
    pulse_rx: Mutex<Receiver<Pulse>>,   // slow‑pulse every ~2 s
}

impl Logger {
    /* ---------------- construction + background loop -------------------- */
    pub fn new(core_rx: Receiver<CoreEvent>, pulse_subs: Arc<PulseSubscriptions>) -> Self {
        Self {
            buffer_logs: RwLock::new(Vec::new()),
            core_event_rx: Mutex::new(core_rx),
            pulse_rx: Mutex::new(pulse_subs.subscribe_slow()),
        }
    }

    /// Spawn this on Tokio runtime: `tokio::spawn(logger.clone().init())`.
    pub async fn init(self: Arc<Self>) {
        let mut core_rx = self.core_event_rx.lock().await;
        let mut pulse_rx = self.pulse_rx.lock().await;
        loop {
            select! {
                Ok(ev) = core_rx.recv() => match ev {
                    CoreEvent::Startup => {},
                    CoreEvent::Restart => { self.store_all_logs().await; },
                    CoreEvent::Shutdown => { self.store_all_logs().await; break; },
                },
                Ok(pulse) = pulse_rx.recv() => {
                    if matches!(pulse, Pulse::Slow) {
                        self.try_clean().await;
                        self.store_all_logs().await;
                    }
                }
            }
        }
    }

    /* ---------------- public API --------------------------------------- */
    #[allow(clippy::too_many_arguments)]
    pub async fn log(
        logger: Arc<Self>, // accept owned Arc to match call sites
        level: LogLevelEnum,
        module: SystemModuleEnum,
        action: LogActionEnum,
        client: Option<ClientConnectedPayload>,
        submitted: Option<JobSubmittedPayload>,
        completed: Option<JobCompletedPayload>,
        custom: Option<String>,
    ) {
        let now = Utc::now().naive_utc();
        let expires_at = match level {
            LogLevelEnum::Info => now + chrono::Duration::minutes(5),
            LogLevelEnum::Success => now + chrono::Duration::days(1),
            LogLevelEnum::Warning => now + chrono::Duration::days(3),
            LogLevelEnum::Error | LogLevelEnum::Fatal => now + chrono::Duration::days(7),
        };
        logger.buffer_logs.write().await.push(LogEntry {
            id: 0,
            created_at: now,
            level,
            module,
            action,
            expires_at,
            client_connected_payload: client,
            job_submitted_payload: submitted,
            job_completed_payload: completed,
            custom_msg: custom,
        });
    }

    /* ---------------- internal helpers --------------------------------- */

    /// Flush buffer → DB (best‑effort).
    pub async fn store_all_logs(&self) {
        let mut pending: Vec<LogEntry> = {
            let mut guard = self.buffer_logs.write().await;
            guard.drain(..).collect()
        };
        if pending.is_empty() {
            return;
        }
        println!("Logger: flushing {} entries to DB…", pending.len());

        match Self::insert_batch(&mut pending).await {
            Ok(_) => println!("Logger: flush OK"),
            Err(e) => {
                eprintln!("Logger: flush failed – returning to buffer: {e}");
                self.buffer_logs.write().await.extend(pending);
            }
        }
    }

    /// Delete expired rows from DB once per slow‑pulse.
    pub async fn try_clean(&self) {
        let mut conn = match load_db_connection().await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Logger: cleanup DB connect error: {e}");
                return;
            }
        };
        let now: NaiveDateTime = Utc::now().naive_utc();
        use common::database::schema::logs::dsl as l;
        if let Err(e) = diesel::delete(l::logs.filter(l::expires_at.lt(now)))
            .execute(&mut conn)
            .await
        {
            eprintln!("Logger: cleanup delete error: {e}");
        }
    }

    async fn insert_batch(entries: &[LogEntry]) -> anyhow::Result<()> {
        let mut conn: AsyncPgConnection = load_db_connection().await?;
        for e in entries {
            let new_row: NewDBLogEntry = e.into();
            let _ = LogEntryRepository::create(&mut conn, new_row).await;
        }
        Ok(())
    }
}
