use std::hash;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum CoreEvent {
    Startup,
    Shutdown,
    Restart,
}

impl ToString for CoreEvent {
    fn to_string(&self) -> String {
        match self {
            CoreEvent::Startup => "Startup".to_string(),
            CoreEvent::Shutdown => "Shutdown".to_string(),
            CoreEvent::Restart => "Restart".to_string(),
        }
    }
}

impl FromStr for CoreEvent {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Startup" => Ok(CoreEvent::Startup),
            "Shutdown" => Ok(CoreEvent::Shutdown),
            "Restart" => Ok(CoreEvent::Restart),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Pulse {
    Slow,   // 1 minute
    Medium, // 1 second
    Fast,   // 50ms
}

impl ToString for Pulse {
    fn to_string(&self) -> String {
        match self {
            Pulse::Slow => "Slow".to_string(),
            Pulse::Medium => "Medium".to_string(),
            Pulse::Fast => "Fast".to_string(),
        }
    }
}

impl FromStr for Pulse {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Slow" => Ok(Pulse::Slow),
            "Medium" => Ok(Pulse::Medium),
            "Fast" => Ok(Pulse::Fast),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum SystemModule {
    Dispatcher,
    Harvester,
    Hibernator,
    Reciever,
    Scheduler,
    TaskArchive,
}

impl ToString for SystemModule {
    fn to_string(&self) -> String {
        match self {
            SystemModule::Dispatcher => "Dispatcher".to_string(),
            SystemModule::Harvester => "Harvester".to_string(),
            SystemModule::Hibernator => "Hibernator".to_string(),
            SystemModule::Reciever => "Reciever".to_string(),
            SystemModule::Scheduler => "Scheduler".to_string(),
            SystemModule::TaskArchive => "TaskArchive".to_string(),
        }
    }
}

impl FromStr for SystemModule {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Dispatcher" => Ok(SystemModule::Dispatcher),
            "Harvester" => Ok(SystemModule::Harvester),
            "Hibernator" => Ok(SystemModule::Hibernator),
            "Reciever" => Ok(SystemModule::Reciever),
            "Scheduler" => Ok(SystemModule::Scheduler),
            "TaskArchive" => Ok(SystemModule::TaskArchive),
            _ => Err(()),
        }
    }
}
