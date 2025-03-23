use crate::enums::System::CoreEvent;
use crate::service_channels::ServiceChannels;
use crate::services::{
    dispatcher::Dispatcher, harvester::Harvester, hibernator::Hibernator, logger::Logger,
    producer_core::ProducerCore, reciever::Reciever, scheduler::Scheduler,
    task_archive::TaskArchive, tcp_authenticator::TCPAuthenticator,
    transmitted_terminal::TransmittedTerminal,
};
use std::mem;
use tokio::sync::{broadcast, mpsc};
use tokio::task;

pub struct ServiceHandles {
    pub dispatcher_task: task::JoinHandle<()>,
    pub harvester_task: task::JoinHandle<()>,
    pub hibernator_task: task::JoinHandle<()>,
    pub logger_task: task::JoinHandle<()>,
    pub reciever_task: task::JoinHandle<()>,
    pub scheduler_task: task::JoinHandle<()>,
    pub tcp_authenticator_task: task::JoinHandle<()>,
    pub transmitted_terminal_task: task::JoinHandle<()>,
    pub producer_core_task: task::JoinHandle<()>,
    pub task_archive_task: task::JoinHandle<()>,
}

impl ServiceHandles {
    pub fn new(mut service_channels: ServiceChannels) -> (Self, ServiceChannels) {
        // Move the core_event_manip_tx out of service_channels
        let (mut moved_tx, _) = mpsc::unbounded_channel::<CoreEvent>();
        mem::swap(&mut moved_tx, &mut service_channels.core_event_manip_tx);

        // Move the necessary core_event_subs out of service_channels

        let dispatcher_task: task::JoinHandle<()> =
            task::spawn(Dispatcher::init(service_channels.core_event_subs.remove(0)));
        let harvester_task: task::JoinHandle<()> =
            task::spawn(Harvester::init(service_channels.core_event_subs.remove(0)));
        let hibernator_task: task::JoinHandle<()> =
            task::spawn(Hibernator::init(service_channels.core_event_subs.remove(0)));
        let logger_task: task::JoinHandle<()> =
            task::spawn(Logger::init(service_channels.core_event_subs.remove(0)));
        let reciever_task: task::JoinHandle<()> =
            task::spawn(Reciever::init(service_channels.core_event_subs.remove(0)));
        let scheduler_task: task::JoinHandle<()> =
            task::spawn(Scheduler::init(service_channels.core_event_subs.remove(0)));
        let tcp_authenticator_task: task::JoinHandle<()> = task::spawn(TCPAuthenticator::init(
            service_channels.core_event_subs.remove(0),
        ));

        let transmitted_terminal_task: task::JoinHandle<()> =
            task::spawn(TransmittedTerminal::init(
                service_channels.core_event_subs.remove(0),
                moved_tx, //
            ));

        let producer_core_task: task::JoinHandle<()> = task::spawn(ProducerCore::init(
            service_channels.core_event_subs.remove(0),
        ));
        let task_archive_task: task::JoinHandle<()> = task::spawn(TaskArchive::init(
            service_channels.core_event_subs.remove(0),
        ));

        let handles = ServiceHandles {
            dispatcher_task,
            harvester_task,
            hibernator_task,
            logger_task,
            reciever_task,
            scheduler_task,
            tcp_authenticator_task,
            transmitted_terminal_task,
            producer_core_task,
            task_archive_task,
        };

        // Return the handles and the modified service_channels
        (handles, service_channels)
    }

    pub async fn join_tasks(self) {
        let _ = tokio::join!(
            self.dispatcher_task,
            self.harvester_task,
            self.hibernator_task,
            self.logger_task,
            self.reciever_task,
            self.scheduler_task,
            self.tcp_authenticator_task,
            self.transmitted_terminal_task,
            self.producer_core_task,
            self.task_archive_task,
        );
    }
}
