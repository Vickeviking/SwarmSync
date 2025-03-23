use crate::enums::System::CoreEvent;
use crate::services::{
    dispatcher::Dispatcher, harvester::Harvester, hibernator::Hibernator, logger::Logger,
    producer_core::ProducerCore, reciever::Reciever, scheduler::Scheduler,
    task_archive::TaskArchive, tcp_authenticator::TCPAuthenticator,
    transmitted_terminal::TransmittedTerminal,
};
use tokio::sync::{broadcast, mpsc};
use tokio::task;

pub mod db;
pub mod enums;
pub mod models;
pub mod services;

#[tokio::main]
async fn main() {
    // ==== Create a broadcast channel for CoreEvent messages =====
    let (core_event_tx, _) = broadcast::channel::<CoreEvent>(16); // Buffer of 16 messages

    // Spawn tasks for each service, giving each one a **separate subscriber**
    let coreEvent_dispatcher_rx = core_event_tx.subscribe();
    let coreEvent_harvester_rx = core_event_tx.subscribe();
    let coreEvent_hibernator_rx = core_event_tx.subscribe();
    let coreEvent_logger_rx = core_event_tx.subscribe();
    let coreEvent_reciever_rx = core_event_tx.subscribe();
    let coreEvent_scheduler_rx = core_event_tx.subscribe();
    let coreEvent_tcp_authenticator_rx = core_event_tx.subscribe();
    let coreEvent_transmitted_terminal_rx = core_event_tx.subscribe();
    let coreEvent_task_archive_rx = core_event_tx.subscribe();
    let coreEvent_producer_core_rx = core_event_tx.subscribe();

    // for Admin terminal to manip the system status
    let (core_event_manip_tx, mut core_event_manip_rx) = mpsc::unbounded_channel::<CoreEvent>();

    // ==== Spawn each service as a separate Tokio task ======
    let dispatcher_task: task::JoinHandle<()> =
        task::spawn(Dispatcher::init(coreEvent_dispatcher_rx));
    let harvester_task: task::JoinHandle<()> = task::spawn(Harvester::init(coreEvent_harvester_rx));
    let hibernator_task: task::JoinHandle<()> =
        task::spawn(Hibernator::init(coreEvent_hibernator_rx));
    let logger_task: task::JoinHandle<()> = task::spawn(Logger::init(coreEvent_logger_rx));
    let reciever_task: task::JoinHandle<()> = task::spawn(Reciever::init(coreEvent_reciever_rx));
    let scheduler_task: task::JoinHandle<()> = task::spawn(Scheduler::init(coreEvent_scheduler_rx));
    let tcp_authenticator_task: task::JoinHandle<()> =
        task::spawn(TCPAuthenticator::init(coreEvent_tcp_authenticator_rx));
    let transmitted_terminal_task: task::JoinHandle<()> = task::spawn(TransmittedTerminal::init(
        coreEvent_transmitted_terminal_rx,
        core_event_manip_tx,
    ));
    let producer_core_task: task::JoinHandle<()> =
        task::spawn(ProducerCore::init(coreEvent_producer_core_rx));
    let task_archive_task: task::JoinHandle<()> =
        task::spawn(TaskArchive::init(coreEvent_task_archive_rx));

    // ==== Send Startup signal to everyone =====
    let _ = core_event_tx.send(CoreEvent::Startup);

    println!("System started. Awaiting commands...");

    // ==== Loop to handle incoming system manipulation events ====
    while let Some(event) = core_event_manip_rx.recv().await {
        match event {
            CoreEvent::Shutdown => {
                println!("Shutdown command received. Notifying all tasks...");

                // Send shutdown signal to all services
                let _ = core_event_tx.send(CoreEvent::Shutdown);

                // Wait for all tasks to complete
                let _ = tokio::join!(
                    dispatcher_task,
                    harvester_task,
                    hibernator_task,
                    logger_task,
                    reciever_task,
                    scheduler_task,
                    tcp_authenticator_task,
                    transmitted_terminal_task,
                    producer_core_task,
                    task_archive_task
                );

                println!("All tasks have finished. System shutdown complete.");
                break; // Exit the loop to shut down main
            }
            CoreEvent::Restart => {
                println!("Restart command received. Notifying all tasks...");
                let _ = core_event_tx.send(CoreEvent::Restart);
            }
            CoreEvent::Startup => {
                println!("Startup event received again. Ignoring...");
            }
        }
    }
}
