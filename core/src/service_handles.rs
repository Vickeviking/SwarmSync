use crate::services::{
    dispatcher::Dispatcher, harvester::Harvester, hibernator::Hibernator, logger::Logger,
    reciever::Reciever, scheduler::Scheduler, task_archive::TaskArchive,
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
    pub task_archive_task: task::JoinHandle<()>,
}

impl ServiceHandles {
    // Synchronous constructor
    pub fn new(shared_resources: Arc<SharedResources>) -> Self {
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

        let task_archive = TaskArchive::new(Arc::clone(&shared_resources));
        let task_archive_task: task::JoinHandle<()> = task::spawn(task_archive.init());

        ServiceHandles {
            dispatcher_task,
            harvester_task,
            hibernator_task,
            logger_task,
            reciever_task,
            scheduler_task,
            task_archive_task,
        }
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
            self.task_archive_task,
        );
    }
}
