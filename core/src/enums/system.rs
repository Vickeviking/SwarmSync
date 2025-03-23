use std::hash::Hash;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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

#[derive(Debug, Clone)]
pub enum Pulse {
    Slow,   // 1 minute
    Medium, // 1 second
    Fast,   // 50ms
}
