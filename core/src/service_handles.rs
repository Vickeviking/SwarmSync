use crate::db::db::Db;
use crate::services::{
    core_bridge::CoreBridge, dispatcher::Dispatcher, harvester::Harvester, hibernator::Hibernator,
    logger::Logger, reciever::Reciever, scheduler::Scheduler, task_archive::TaskArchive,
    tcp_authenticator::TCPAuthenticator,
};
use crate::shared_resources::SharedResources;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::Notify;
use tokio::task;

pub struct ServiceHandles {
    pub dispatcher_task: task::JoinHandle<()>,
    pub harvester_task: task::JoinHandle<()>,
    pub hibernator_task: task::JoinHandle<()>,
    pub logger_task: task::JoinHandle<()>,
    pub reciever_task: task::JoinHandle<()>,
    pub scheduler_task: task::JoinHandle<()>,
    pub tcp_authenticator_task: task::JoinHandle<()>,
    pub core_bridge_task: task::JoinHandle<()>,
    pub grpc_task: task::JoinHandle<()>,
    pub task_archive_task: task::JoinHandle<()>,
    pub db_task: task::JoinHandle<()>,
}

impl ServiceHandles {
    // Synchronous constructor
    pub fn new(shared_resources: Arc<SharedResources>) -> (Self, CoreBridge) {
        // Create the dispatcher, harvester, etc., synchronously.
        let dispatcher = Dispatcher::new(Arc::clone(&shared_resources));
        let dispatcher_task: task::JoinHandle<()> = task::spawn(dispatcher.init());

        let harvester = Harvester::new(Arc::clone(&shared_resources));
        let harvester_task: task::JoinHandle<()> = task::spawn(harvester.init());

        let hibernator = Hibernator::new(Arc::clone(&shared_resources));
        let hibernator_task: task::JoinHandle<()> = task::spawn(hibernator.init());

        let logger_task: task::JoinHandle<()> =
            task::spawn(Logger::init(shared_resources.get_logger()));

        let reciever = Reciever::new(Arc::clone(&shared_resources));
        let reciever_task: task::JoinHandle<()> = task::spawn(reciever.init());

        let scheduler = Scheduler::new(Arc::clone(&shared_resources));
        let scheduler_task: task::JoinHandle<()> = task::spawn(scheduler.init());

        let tcp_authenticator = TCPAuthenticator::new(Arc::clone(&shared_resources));
        let tcp_authenticator_task: task::JoinHandle<()> = task::spawn(tcp_authenticator.init());

        // CoreBridge initialization will be handled asynchronously later
        let core_bridge_instance = CoreBridge::new(Arc::clone(&shared_resources));

        // Task Archive and DB initialization as well
        let task_archive = TaskArchive::new(Arc::clone(&shared_resources));
        let task_archive_task: task::JoinHandle<()> = task::spawn(task_archive.init());

        let db = Db::new(Arc::clone(&shared_resources));
        let db_task: task::JoinHandle<()> = task::spawn(db.init());

        let handles = ServiceHandles {
            dispatcher_task,
            harvester_task,
            hibernator_task,
            logger_task,
            reciever_task,
            scheduler_task,
            tcp_authenticator_task,
            core_bridge_task: task::spawn(async {}), //placeholder task
            grpc_task: task::spawn(async {}),        // Placeholder task for now
            task_archive_task,
            db_task,
        };
        (handles, core_bridge_instance)
    }

    // Async initialization method for CoreBridge
    pub async fn initialize_core_bridge(
        &mut self,
        shared_resources: Arc<SharedResources>,
        mut core_bridge: CoreBridge,
    ) -> Result<(), String> {
        let notify = Arc::new(Notify::new());

        // Wire the channels asynchronously
        core_bridge = core_bridge.wire_channels().await?;

        let core_bridge_arc = Arc::new(Mutex::new(core_bridge));

        // Start CoreBridge and gRPC server tasks
        let core_bridge_task: task::JoinHandle<()> = tokio::spawn(CoreBridge::init(
            shared_resources
                .get_service_channels()
                .subscribe_to_core_event(),
            Arc::clone(&notify), // Pass notify here
        ));

        let grpc_task: task::JoinHandle<()> = tokio::spawn(CoreBridge::start_grpc_server(
            Arc::clone(&core_bridge_arc),
            Arc::clone(&notify), // Pass notify here
        ));

        self.core_bridge_task = core_bridge_task;
        self.grpc_task = grpc_task;

        Ok(())
    }

    // Join all tasks
    pub async fn join_tasks(self) {
        let _ = tokio::join!(
            self.dispatcher_task,
            self.harvester_task,
            self.hibernator_task,
            self.logger_task,
            self.reciever_task,
            self.scheduler_task,
            self.tcp_authenticator_task,
            self.core_bridge_task,
            self.grpc_task,
            self.task_archive_task,
            self.db_task,
        );
    }
}
