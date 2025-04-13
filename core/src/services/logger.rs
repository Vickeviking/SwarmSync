use crate::enums::system::{CoreEvent, Pulse, SystemModule};
use crate::pulse_broadcaster::PulseBroadcaster;
use crate::pulse_broadcaster::PulseSubscriptions;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::select;
use tokio::sync::Mutex;
use tokio::sync::{broadcast::Receiver, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum LogLevel {
    Info,    //Expire in 5 minutes
    Success, //Expire in 1 day
    Warning, //Expire in 3 days
    Error,   // Expire in 7 days
    Fatal,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum LogAction {
    ClientConnected {
        ip: String,
        username: String,
    },
    JobSubmitted {
        job_id: Uuid,
        from: SystemModule,
        to: SystemModule,
    },
    JobCompleted {
        job_id: Uuid,
        success: bool,
    },
    SystemStarted,
    SystemShutdown,
    Custom(String),
}

pub struct Logger {
    logs: RwLock<HashMap<Uuid, LogEntry>>,
    module_logs: RwLock<HashMap<SystemModule, Vec<Uuid>>>,
    core_event_rx: Mutex<Receiver<CoreEvent>>,
    pulse_rx: Mutex<Receiver<Pulse>>,
}

#[derive(Clone)]
pub struct LogEntry {
    timestamp: DateTime<Utc>,
    level: LogLevel,
    module: SystemModule,
    action: LogAction,
    expires_at: DateTime<Utc>,
}

impl Logger {
    pub fn new(
        core_event_rx: Receiver<CoreEvent>,
        pulse_subscriptions: Arc<PulseSubscriptions>,
    ) -> Self {
        let pulse_rx = pulse_subscriptions.subscribe_slow();

        Logger {
            logs: RwLock::new(HashMap::new()),
            module_logs: RwLock::new(HashMap::new()),
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
                            Logger::load_all_logs().await;
                         },
                         CoreEvent::Restart => {
                            println!("Logger: Restart event received.");
                            Logger::store_all_logs().await;
                            Logger::load_all_logs().await;
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

    pub async fn log(
        logger: Arc<Logger>,
        log_level: LogLevel,
        system_module: SystemModule,
        log_action: LogAction,
    ) {
        let timestamp = Utc::now();
        let expires_at = match log_level {
            LogLevel::Info => timestamp + chrono::Duration::minutes(5),
            LogLevel::Success => timestamp + chrono::Duration::days(1),
            LogLevel::Warning => timestamp + chrono::Duration::days(3),
            LogLevel::Error => timestamp + chrono::Duration::days(7),
            LogLevel::Fatal => timestamp + chrono::Duration::days(7),
        };

        let log_entry = LogEntry {
            timestamp,
            level: log_level,
            module: system_module.clone(),
            action: log_action,
            expires_at,
        };

        let log_id = Uuid::new_v4(); // Generate a new UUID for the log

        // Store the log entry in the `logs` HashMap
        {
            let mut logs = logger.logs.write().await;
            logs.insert(log_id, log_entry);
        }

        // Update `module_logs` to associate the log's UUID with the system module
        {
            let mut module_logs = logger.module_logs.write().await;
            let entry = module_logs.entry(system_module).or_insert_with(Vec::new);
            entry.push(log_id);
        }
    }

    pub async fn get_logs(
        &self,
        module: Option<SystemModule>,
        level: Option<LogLevel>,
        action_filter: Option<&LogAction>,
        since: Option<DateTime<Utc>>,
    ) -> Vec<LogEntry> {
        let logs = self.logs.read().await;

        let mut filtered_logs: Vec<LogEntry> = logs
            .values()
            .filter(|log| {
                let matches_module = module.as_ref().is_none_or(|m| &log.module == m);
                let matches_level = level.as_ref().is_none_or(|l| &log.level == l);
                let matches_action = action_filter.is_none_or(|a| log.action == *a);
                let matches_since = since.is_none_or(|t| log.timestamp >= t);
                matches_module && matches_level && matches_action && matches_since
            })
            .cloned()
            .collect();
        filtered_logs.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        filtered_logs
    }

    pub async fn try_clean() {
        println!("Tried cleaning");
    }

    pub async fn store_all_logs() {
        println!("Storing logs..");
        //TODO: store logs
    }

    pub async fn load_all_logs() {
        println!("Loading from DB..");
        //TODO: load logs from DB
    }

    pub async fn store_log(entry: &LogEntry) {
        todo!()
    }

    pub async fn store_logs(entry: Vec<&LogEntry>) {
        todo!()
    }
}

// Your existing Logger implementation...

#[cfg(test)]
mod tests {
    use super::*; // Import everything from the outer module
    use tokio::sync::broadcast;

    #[tokio::test]
    async fn test_logger_initialization() {
        assert!(true)
    }

    #[tokio::test]
    async fn test_logging() {
        assert!(true)
    }
}
