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
    pub core_bridge: task::JoinHandle<Result<(), String>>,
    pub grpc_task: task::JoinHandle<()>,
    pub task_archive_task: task::JoinHandle<()>,
    pub db_task: task::JoinHandle<()>,
}

impl ServiceHandles {
    pub fn new(shared_resources: Arc<SharedResources>) -> Self {
        // ====== DISPATCHER ======
        let dispatcher: Dispatcher = Dispatcher::new(Arc::clone(&shared_resources));
        let dispatcher_task: task::JoinHandle<()> = task::spawn(dispatcher.init());

        // ====== HARVESTER ======
        let harvester = Harvester::new(Arc::clone(&shared_resources));
        let harvester_task: task::JoinHandle<()> = task::spawn(harvester.init());

        // ====== HIBERNATOR ======
        let hibernator = Hibernator::new(Arc::clone(&shared_resources));
        let hibernator_task: task::JoinHandle<()> = task::spawn(hibernator.init());

        // ====== LOGGER ======
        let logger_task: task::JoinHandle<()> =
            task::spawn(Logger::init(shared_resources.get_logger()));

        // ====== RECIEVER ======
        let reciever = Reciever::new(Arc::clone(&shared_resources));
        let reciever_task: task::JoinHandle<()> = task::spawn(reciever.init());

        // ====== SCHEDULER ======
        let scheduler = Scheduler::new(Arc::clone(&shared_resources));
        let scheduler_task: task::JoinHandle<()> = task::spawn(scheduler.init());

        // ====== TCP AUTHENTICATOR ======
        let tcp_authenticator = TCPAuthenticator::new(Arc::clone(&shared_resources));
        let tcp_authenticator_task: task::JoinHandle<()> = task::spawn(tcp_authenticator.init());

        // ====== CORE BRIDGE ======
        //used inside corebridge to send shutdown to gRPC server
        let notify = Arc::new(Notify::new());
        // Initialize the CoreBridge instance
        let core_bridge_instance = CoreBridge::new(Arc::clone(&shared_resources));
        let core_bridge_arc = Arc::new(Mutex::new(core_bridge_instance));

        // Spawn the init task and pass notify to it
        let init_task: task::JoinHandle<Result<(), String>> = tokio::spawn(CoreBridge::init(
            shared_resources
                .get_service_channels()
                .subscribe_to_core_event(),
            Arc::clone(&notify), // Pass notify here
        ));

        // Spawn the gRPC server task and pass notify to it
        let grpc_task: task::JoinHandle<()> = tokio::spawn(CoreBridge::start_grpc_server(
            Arc::clone(&core_bridge_arc),
            Arc::clone(&notify), // Pass notify here
        ));
        // ====== TASK ARCHIVE ======
        let task_archive = TaskArchive::new(Arc::clone(&shared_resources));
        let task_archive_task: task::JoinHandle<()> = task::spawn(task_archive.init());

        // ====== DB ======
        let db = Db::new(Arc::clone(&shared_resources));
        let db_task: task::JoinHandle<()> = task::spawn(db.init());

        ServiceHandles {
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
        }
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
