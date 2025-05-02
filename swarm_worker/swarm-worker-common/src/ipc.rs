//! IPC‑level constants shared by the worker binary and front‑ends.
pub const SHUTDOWN_SOCKET: &str = "/tmp/swarm_worker_shutdown.sock";
pub const WORKER_LOG: &str = "/tmp/swarm_worker.log";
pub const UDP_HEARTBEAT_PORT: u32 = 5001;
