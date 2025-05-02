use anyhow::{anyhow, Context};
use rand::Rng;
use std::{fs, sync::Arc, thread::available_parallelism};
use swarm_worker_common::config;
use swarm_worker_common::ipc::{SHUTDOWN_SOCKET, UDP_HEARTBEAT_PORT};
use tokio::{
    io::AsyncReadExt,
    net::{UdpSocket, UnixListener},
    select,
    sync::{broadcast, mpsc, Mutex},
    task,
    time::{sleep, Duration},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    // --------------------------------------------------
    // Load configuration
    // --------------------------------------------------
    let core_config =
        config::load_core_config().context("Could not load in config, needed to resolve host")?;
    let core_ip = core_config.base_url; // assumed to be plain IP/host string
    let worker_id_option = core_config.worker_id; // assumed to be string

    // --------------------------------------------------
    // Shared shutdown broadcast
    // --------------------------------------------------
    let (tx_shutdown, _) = broadcast::channel::<()>(1);

    // --------------------------------------------------
    // Spawn heartbeat task (UDP, 50 ms interval)
    // --------------------------------------------------
    {
        let core_ip = core_ip.clone();
        let worker_id: i32 = worker_id_option
            .ok_or_else(|| anyhow!("worker_id not retrievable from config file"))?;
        let mut rx_shutdown = tx_shutdown.subscribe();
        task::spawn(async move {
            let remote = format!("{}:{}", core_ip, UDP_HEARTBEAT_PORT);
            let socket = match UdpSocket::bind("0.0.0.0:0").await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("heartbeat socket bind failed: {e}");
                    return;
                }
            };

            loop {
                select! {
                    _ = rx_shutdown.recv() => break,   // stop on shutdown
                    _ = async {
                        let payload = worker_id.to_string();
                        let _ = socket.send_to(payload.as_bytes(), &remote).await;
                        sleep(Duration::from_millis(50)).await;
                    } => {}
                }
            }
        });
    }

    // --------------------------------------------------
    // Socket listener for shutdown
    // --------------------------------------------------
    tokio::spawn(socket_listener(tx_shutdown.clone()));

    // --------------------------------------------------
    // Job channel + worker pool
    // --------------------------------------------------
    let (tx_jobs, rx_jobs) = mpsc::channel::<Job>(1024);
    let rx_jobs = Arc::new(Mutex::new(rx_jobs));

    let n_workers = available_parallelism()?.get();
    for _ in 0..n_workers {
        let rx_jobs = Arc::clone(&rx_jobs);
        let mut rx_shutdown = tx_shutdown.subscribe();

        task::spawn(async move {
            loop {
                select! {
                    _ = rx_shutdown.recv() => break,
                    job = async {
                        let mut guard = rx_jobs.lock().await;
                        guard.recv().await
                    } => {
                        if let Some(job) = job { process(job).await }
                        else { break }
                    }
                }
            }
        });
    }

    // --------------------------------------------------
    // Main producer loop
    // --------------------------------------------------
    let mut rx_shutdown = tx_shutdown.subscribe();
    loop {
        select! {
            _ = rx_shutdown.recv() => break,
            maybe_job = next_job() => {
                if let Some(job) = maybe_job {
                    let _ = tx_jobs.send(job).await;
                } else {
                    sleep(Duration::from_millis(50)).await;
                }
            }
        }
    }

    println!("producer exiting – waiting for workers to drain…");
    drop(tx_jobs);
    sleep(Duration::from_secs(1)).await;
    println!("worker shut down gracefully");

    // Clean up socket file synchronously
    let _ = fs::remove_file(SHUTDOWN_SOCKET);
    Ok(())
}

// --------------------------------------------------
// Shutdown socket listener (Unix domain)
// --------------------------------------------------
async fn socket_listener(tx: broadcast::Sender<()>) -> anyhow::Result<()> {
    // Remove stale socket synchronously
    if fs::metadata(SHUTDOWN_SOCKET).is_ok() {
        let _ = fs::remove_file(SHUTDOWN_SOCKET);
    }

    let listener = UnixListener::bind(SHUTDOWN_SOCKET)?;
    println!("Listening on {} (async)", SHUTDOWN_SOCKET);

    let (mut stream, _) = listener.accept().await?;
    let mut buf = Vec::new();
    let _ = stream.read_to_end(&mut buf).await;
    println!("shutdown requested via socket");
    let _ = tx.send(());
    Ok(())
}

// --------------------------------------------------
// Demo job API
// --------------------------------------------------
struct Job(u64);

async fn next_job() -> Option<Job> {
    // 1‑in‑20 chance of Some(Job)
    if rand::thread_rng().gen_ratio(1, 20) {
        static mut COUNTER: u64 = 0;
        unsafe {
            COUNTER += 1;
            Some(Job(COUNTER))
        }
    } else {
        None
    }
}

async fn process(job: Job) {
    println!("processed job {}", job.0);
}
