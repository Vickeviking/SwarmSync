///! Module initializer, spawns all modules async.
use std::sync::Arc;
use tokio::task;

use crate::core::shared_resources::SharedResources;
use crate::modules::logger::Logger;
use crate::modules::{Dispatcher, Harvester, Hibernator, Reciever, Scheduler, TaskArchive};
use crate::rocket_api;

/// Holds tokio handles to all modules
pub struct ModuleInitializer {
    pub dispatcher_task: task::JoinHandle<anyhow::Result<(), anyhow::Error>>,
    pub harvester_task: task::JoinHandle<()>,
    pub hibernator_task: task::JoinHandle<()>,
    pub logger_task: task::JoinHandle<()>,
    pub reciever_task: task::JoinHandle<()>,
    pub scheduler_task: task::JoinHandle<()>,
    pub task_archive_task: task::JoinHandle<()>,
    pub rocket_task: task::JoinHandle<()>,
}

impl ModuleInitializer {
    // Create a new ModuleInitializer with a shared_resource
    pub fn new(shared_resources: Arc<SharedResources>) -> Self {
        let dispatcher = Dispatcher::new(Arc::clone(&shared_resources));
        let dispatcher_task = task::spawn(dispatcher.init());

        let harvester = Harvester::new(Arc::clone(&shared_resources));
        let harvester_task = task::spawn(harvester.init());

        let hibernator = Hibernator::new(Arc::clone(&shared_resources));
        let hibernator_task = task::spawn(hibernator.init());

        let logger = Logger::init(shared_resources.get_logger());
        let logger_task = task::spawn(logger);

        let reciever = Reciever::new(Arc::clone(&shared_resources));
        let reciever_task = task::spawn(reciever.init());

        let scheduler = Scheduler::new(Arc::clone(&shared_resources));
        let scheduler_task = task::spawn(scheduler.init());

        let task_archive = TaskArchive::new(Arc::clone(&shared_resources));
        let task_archive_task = task::spawn(task_archive.init());

        // The rocket server
        let rocket_task = task::spawn(async move {
            rocket_api::rocket_server::launch_rocket(Arc::clone(&shared_resources)).await;
        });

        ModuleInitializer {
            dispatcher_task,
            harvester_task,
            hibernator_task,
            logger_task,
            reciever_task,
            scheduler_task,
            task_archive_task,
            rocket_task,
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
            self.task_archive_task,
            self.rocket_task
        );
    }
}
