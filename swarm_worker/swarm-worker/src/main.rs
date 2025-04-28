use anyhow::Result;
use clap::Parser;
use signal_hook::consts::SIGTERM;
use signal_hook::flag;
use std::{
    env,
    net::UdpSocket,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

#[derive(Parser)]
struct Opt {
    /// Path to worker_config.json
    #[clap(long, default_value = "worker_config.json")]
    config: String,
}

fn main() -> Result<()> {
    let opt = Opt::parse();
    // 1) Load config (here we skip—just echo the path)
    println!("Using config: {}", opt.config);

    // 2) Detect local IP by “connecting” a UDP socket to Core:5001
    let core_addr = "127.0.0.1:5001"; // TODO: parse host from config
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect(core_addr)?;
    let local_ip = socket.local_addr()?.ip();
    println!(
        "Please register this worker in CommandDeck with IP {}",
        local_ip
    );

    // 3) Install a SIGTERM handler
    let term_flag = Arc::new(AtomicBool::new(false));
    flag::register(SIGTERM, Arc::clone(&term_flag))?;

    // 4) Run until we see TERM
    while !term_flag.load(Ordering::Relaxed) {
        // In real worker you'd heartbeat, poll jobs, etc.
        thread::sleep(Duration::from_secs(1));
    }

    println!("Received shutdown signal, exiting cleanly.");
    Ok(())
}
