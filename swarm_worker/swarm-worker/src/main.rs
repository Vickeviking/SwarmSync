use anyhow::{anyhow, Context, Result};
use rand::Rng;
use std::{fs, sync::Arc, thread::available_parallelism};
use swarm_worker_common::config;
use swarm_worker_common::ipc::{CORE_UDP_HEARTBEAT_PORT, SHUTDOWN_SOCKET};
use tokio::{
    io::AsyncReadExt,
    net::{UdpSocket, UnixListener},
    select,
    sync::{broadcast, mpsc, Mutex},
    task,
    time::{sleep, Duration},
};
use url::Url;

/// Extract just the host name (e.g. "core") from CORE_BASE_URL so we can
/// build a clean "host:port" string for UDP. Accepts raw hosts ("core")
/// or full URLs ("http://core:8000").
fn udp_host(base_url: &str) -> Result<String> {
    // Try parsing as URL first, otherwise prefix http:// so Url::parse works
    let url = Url::parse(base_url).or_else(|_| Url::parse(&format!("http://{base_url}")))?;

    Ok(url
        .host_str()
        .ok_or_else(|| anyhow!("CORE_BASE_URL missing host component"))?
        .to_owned())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    // ── 1. Load config ────────────────────────────────────────────────
    let core_config = config::load_core_config().context("Could not load core config")?;

    let core_host = udp_host(&core_config.base_url)?; // e.g. "core"

    let worker_id: i32 = core_config
        .worker_id
        .ok_or_else(|| anyhow!("worker_id not in config"))?;

    let (tx_shutdown, _) = broadcast::channel::<()>(1);

    // ── 2. Heart‑beat task (UDP) ──────────────────────────────────────
    {
        let core_addr = format!("{core_host}:{CORE_UDP_HEARTBEAT_PORT}"); // "core:5001"
        let mut rx_shutdown = tx_shutdown.subscribe();

        task::spawn(async move {
            let socket = match UdpSocket::bind("0.0.0.0:0").await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("heartbeat bind failed: {e}");
                    return;
                }
            };

            // 2.1 Send CONNECT once
            let connect_msg = format!("{worker_id},CONNECT");
            match socket.send_to(connect_msg.as_bytes(), &core_addr).await {
                Ok(n) => println!("HB‑TX {n} B -> {core_addr}: {connect_msg}"),
                Err(e) => eprintln!("HB‑ERR failed to send CONNECT to {core_addr}: {e}"),
            }

            // 2.2 Main loop – periodic heartbeat + shutdown handling
            loop {
                select! {
                    // graceful shutdown → DISCONNECT
                    _ = rx_shutdown.recv() => {
                        let disc_msg = format!("{worker_id},DISCONNECT");
                        match socket.send_to(disc_msg.as_bytes(), &core_addr).await {
                            Ok(n) => println!("HB‑TX {n} B -> {core_addr}: {disc_msg}"),
                            Err(e) => eprintln!("HB‑ERR failed to send DISCONNECT: {e}"),
                        }
                        break;
                    }

                    // periodic heartbeat every 50 ms → IDLE/BUSY
                    _ = async {
                        let payload = format!("{worker_id},IDLE");

                        match socket.send_to(payload.as_bytes(), &core_addr).await {
                            Ok(n) => println!("HB‑TX {n} B -> {core_addr}: {payload}"),
                            Err(e) => eprintln!("HB‑ERR send failed: {e}"),
                        };

                        sleep(Duration::from_millis(500)).await;
                    } => {}
                }
            }
        });
    }

    // ── 3. Shutdown socket listener ──────────────────────────────────
    task::spawn(socket_listener(tx_shutdown.clone()));

    // ── 4. Job channel + worker pool ─────────────────────────────────
    let (tx_jobs, rx_jobs) = mpsc::channel::<Job>(1024);
    let rx_jobs = Arc::new(Mutex::new(rx_jobs));
    let n = available_parallelism()?.get();
    for _ in 0..n {
        let rx = Arc::clone(&rx_jobs);
        let mut rx_shutdown = tx_shutdown.subscribe();
        task::spawn(async move {
            loop {
                select! {
                    _ = rx_shutdown.recv() => break,
                    job = async { rx.lock().await.recv().await } => {
                        if let Some(job) = job { process(job).await } else { break }
                    }
                }
            }
        });
    }

    // ── 5. Main producer loop ────────────────────────────────────────
    let mut rx_shutdown = tx_shutdown.subscribe();
    loop {
        select! {
            _ = rx_shutdown.recv() => break,
            maybe_job = next_job() => {
                if let Some(j) = maybe_job { let _ = tx_jobs.send(j).await; }
                else { sleep(Duration::from_millis(50)).await; }
            }
        }
    }

    println!("producer exiting – waiting for workers…");
    drop(tx_jobs);
    sleep(Duration::from_secs(1)).await;
    println!("worker shut down gracefully");
    let _ = fs::remove_file(SHUTDOWN_SOCKET);

    Ok(())
}

async fn socket_listener(tx: broadcast::Sender<()>) -> Result<()> {
    if fs::metadata(SHUTDOWN_SOCKET).is_ok() {
        let _ = fs::remove_file(SHUTDOWN_SOCKET);
    }
    let listener = UnixListener::bind(SHUTDOWN_SOCKET)?;
    let (mut stream, _) = listener.accept().await?;
    let mut buf = Vec::new();
    let _ = stream.read_to_end(&mut buf).await;
    let _ = tx.send(());
    Ok(())
}

struct Job(u64);

/// Generates a random job or none
async fn next_job() -> Option<Job> {
    if rand::thread_rng().gen_ratio(1, 20) {
        static mut C: u64 = 0;
        unsafe {
            C += 1;
            Some(Job(C))
        }
    } else {
        None
    }
}

async fn process(job: Job) {
    println!("processed job {}", job.0);
}

// TODO: write tests here
#[cfg(test)]
mod tests {

    /// A dummy test to ensure your test harness is working.
    #[test]
    fn dummy_test() {
        // Replace this with real assertions as you go
        assert_eq!(2 + 2, 4);
    }
}
