use crate::core::PulseSubscriptions;
use crate::database::models::log::{
    ClientConnectedPayload, JobCompletedPayload, JobSubmittedPayload, LogEntry,
};
use crate::shared::enums::log::{LogActionEnum, LogLevelEnum};
use crate::shared::enums::system::{CoreEvent, Pulse, SystemModuleEnum};
use chrono::Utc;
use std::sync::Arc;
use tokio::select;
use tokio::sync::{broadcast::Receiver, Mutex, RwLock};

pub struct Logger {
    buffer_logs: RwLock<Vec<LogEntry>>, // sent to db on pulse, on convertion use NewDBLogEntry
    core_event_rx: Mutex<Receiver<CoreEvent>>,
    pulse_rx: Mutex<Receiver<Pulse>>,
}

impl Logger {
    pub fn new(
        core_event_rx: Receiver<CoreEvent>,
        pulse_subscriptions: Arc<PulseSubscriptions>,
    ) -> Self {
        let pulse_rx = pulse_subscriptions.subscribe_slow();

        Logger {
            buffer_logs: RwLock::new(Vec::new()),
            core_event_rx: Mutex::new(core_event_rx),
            pulse_rx: Mutex::new(pulse_rx),
        }
    }

    pub async fn init(logger: Arc<Logger>) {
        let mut core_event_rx = logger.core_event_rx.lock().await;
        let mut pulse_rx = logger.pulse_rx.lock().await;
        loop {
            select! {
                Ok(event) = core_event_rx.recv() => {
                    match event {
                        CoreEvent::Startup => {
                            println!("Logger: Startup event received.");
                        },
                        CoreEvent::Restart => {
                            println!("Logger: Restart event received.");
                            Logger::store_all_logs().await;
                        },
                        CoreEvent::Shutdown => {
                            println!("Logger: Shutdown event received. Stopping...");
                            Logger::store_all_logs().await;
                            break;
                        }
                    }
                }
                Ok(pulse) = pulse_rx.recv() => {
                    if let Pulse::Slow = pulse {
                        println!("Logger: pulse received.");
                        Logger::try_clean().await;
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn log(
        logger: Arc<Logger>,
        log_level: LogLevelEnum,
        system_module: SystemModuleEnum,
        log_action: LogActionEnum,
        client_payload: Option<ClientConnectedPayload>,
        submitted_payload: Option<JobSubmittedPayload>,
        completed_payload: Option<JobCompletedPayload>,
        custom_msg: Option<String>,
    ) {
        let now = Utc::now().naive_utc();
        let expires_at = match log_level {
            LogLevelEnum::Info => now + chrono::Duration::minutes(5),
            LogLevelEnum::Success => now + chrono::Duration::days(1),
            LogLevelEnum::Warning => now + chrono::Duration::days(3),
            LogLevelEnum::Error | LogLevelEnum::Fatal => now + chrono::Duration::days(7),
        };

        let log_entry = LogEntry {
            id: 0, // To be populated by DB
            created_at: now,
            level: log_level,
            module: system_module,
            action: log_action,
            expires_at,
            client_connected_payload: client_payload,
            job_submitted_payload: submitted_payload,
            job_completed_payload: completed_payload,
            custom_msg,
        };

        logger.buffer_logs.write().await.push(log_entry);
    }

    pub async fn try_clean() {
        //should load in a heafty amount of logs and start a process of deleting logs that has
        //expired from the database
        println!("Tried cleaning");
    }

    pub async fn store_all_logs() {
        println!("Storing logs..");
        // TODO: store logs
    }

    pub async fn store_log(_entry: &LogEntry) {
        todo!()
        // TODO: store logs
    }

    pub async fn store_logs(_entry: Vec<&LogEntry>) {
        todo!()
        // TODO: store logs
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_logger_initialization() {
        assert!(true)
    }

    #[tokio::test]
    async fn test_logging() {
        assert!(true)
    }
}
