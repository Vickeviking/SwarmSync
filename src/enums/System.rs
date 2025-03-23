pub enum SystemModule {
    TaskReceiver,
    TaskScheduler,
    InteractiveTerminal,
    TaskHibernator,
    TaskDispatcher,
    TCPAuthenticator,
    Harvester,
}

#[derive(Debug, Clone)]
pub enum CoreEvent {
    Startup,
    Shutdown,
    Restart,
}
