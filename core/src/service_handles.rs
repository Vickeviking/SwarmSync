use crate::db::db::Db;
use crate::enums::system::CoreEvent;
use crate::pulse_broadcaster::PulseBroadcaster;
use crate::service_channels::ServiceChannels;
use crate::services::{
    core_bridge::CoreBridge, dispatcher::Dispatcher, harvester::Harvester, hibernator::Hibernator,
    logger::Logger, reciever::Reciever, scheduler::Scheduler, task_archive::TaskArchive,
    tcp_authenticator::TCPAuthenticator,
};
use crate::shared_resources::SharedResources;
use std::mem;
use std::sync::Arc;
use tokio::sync::Mutex;
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
    pub core_bridge: task::JoinHandle<()>,
    pub grpc_task: task::JoinHandle<()>,
    pub task_archive_task: task::JoinHandle<()>,
    pub db_task: task::JoinHandle<()>,
}

impl ServiceHandles {
    pub fn new(
        mut service_channels: ServiceChannels,
        pulse_broadcaster: PulseBroadcaster,
        shared_resources: SharedResources,
    ) -> (Self, ServiceChannels, PulseBroadcaster, SharedResources) {
        // Move the core_event_manip_tx out of service_channels
        let (mut moved_tx, _) = mpsc::unbounded_channel::<CoreEvent>();
        mem::swap(&mut moved_tx, &mut service_channels.core_event_manip_tx);

        // ====== DISPATCHER ======
        let dispatcher_task: task::JoinHandle<()> =
            task::spawn(Dispatcher::init(service_channels.subscribe_to_core_event()));

        // ====== HARVESTER ======
        let harvester_task: task::JoinHandle<()> =
            task::spawn(Harvester::init(service_channels.subscribe_to_core_event()));

        // ====== HIBERNATOR ======
        let hibernator_task: task::JoinHandle<()> =
            task::spawn(Hibernator::init(service_channels.subscribe_to_core_event()));

        // ====== LOGGER ======
        let logger_task: task::JoinHandle<()> =
            task::spawn(Logger::init(shared_resources.logger.clone()));

        // ====== RECIEVER ======
        let reciever_task: task::JoinHandle<()> =
            task::spawn(Reciever::init(service_channels.subscribe_to_core_event()));

        // ====== SCHEDULER ======
        let scheduler_task: task::JoinHandle<()> =
            task::spawn(Scheduler::init(service_channels.subscribe_to_core_event()));

        // ====== TCP AUTHENTICATOR ======
        let tcp_authenticator_task: task::JoinHandle<()> = task::spawn(TCPAuthenticator::init(
            service_channels.subscribe_to_core_event(),
        ));

        // ====== CORE BRIDGE ======
        let core_bridge_instance =
            CoreBridge::new(moved_tx, Some(pulse_broadcaster.subscribe_slow()));
        // Arc<Mutex<CoreBridge>> shared access
        let core_bridge_arc = Arc::new(Mutex::new(core_bridge_instance));

        let init_task: task::JoinHandle<()> = tokio::spawn(CoreBridge::init(
            Arc::clone(&core_bridge_arc),
            service_channels.subscribe_to_core_event(),
        ));

        // Spawn the gRPC server, consuming the core_bridge instance
        let grpc_task: task::JoinHandle<()> =
            tokio::spawn(CoreBridge::start_grpc_server(Arc::clone(&core_bridge_arc)));

        // ====== TASK ARCHIVE ======
        let task_archive_task: task::JoinHandle<()> = task::spawn(TaskArchive::init(
            service_channels.subscribe_to_core_event(),
        ));

        // ====== DB ======
        let db_task: task::JoinHandle<()> =
            task::spawn(Db::init(service_channels.subscribe_to_core_event()));

        let handles = ServiceHandles {
            dispatcher_task,
            harvester_task,
            hibernator_task,
            logger_task,
            reciever_task,
            scheduler_task,
            tcp_authenticator_task,
            core_bridge: init_task,
            grpc_task,
            task_archive_task,
            db_task,
        };

        // Return the handles and the modified service_channels
        (
            handles,
            service_channels,
            pulse_broadcaster,
            shared_resources,
        )
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
            self.core_bridge,
            self.grpc_task,
            self.task_archive_task,
            self.db_task,
        );
    }
}
