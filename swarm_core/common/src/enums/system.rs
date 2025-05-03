use core::fmt;
use diesel::deserialize::{FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::ToSql;
use diesel::sql_types::Text;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum CoreEvent {
    Startup,
    Shutdown,
    Restart,
}

impl fmt::Display for CoreEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            CoreEvent::Startup => "Startup",
            CoreEvent::Shutdown => "Shutdown",
            CoreEvent::Restart => "Restart",
        };
        write!(f, "{}", name)
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

impl fmt::Display for Pulse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Pulse::Slow => "Slow",
            Pulse::Medium => "Medium",
            Pulse::Fast => "Fast",
        };
        write!(f, "{}", s)
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

#[derive(AsExpression, Debug, Deserialize, Serialize, FromSqlRow, Clone)]
#[diesel(sql_type = diesel::sql_types::VarChar)]
pub enum SystemModuleEnum {
    Dispatcher,
    Harvester,
    Hibernator,
    Receiver,
    Scheduler,
    TaskArchive,
}

impl SystemModuleEnum {
    pub fn variants() -> &'static [&'static str] {
        &[
            "Dispatcher",
            "Harvester",
            "Hibernator",
            "Receiver",
            "Scheduler",
            "TaskArchive",
        ]
    }
}

impl From<usize> for SystemModuleEnum {
    fn from(idx: usize) -> Self {
        match idx {
            0 => SystemModuleEnum::Dispatcher,
            1 => SystemModuleEnum::Harvester,
            2 => SystemModuleEnum::Hibernator,
            3 => SystemModuleEnum::Receiver,
            4 => SystemModuleEnum::Scheduler,
            5 => SystemModuleEnum::TaskArchive,
            _ => SystemModuleEnum::Dispatcher,
        }
    }
}

impl fmt::Display for SystemModuleEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SystemModuleEnum::Dispatcher => "Dispatcher",
            SystemModuleEnum::Harvester => "Harvester",
            SystemModuleEnum::Hibernator => "Hibernator",
            SystemModuleEnum::Receiver => "Receiver",
            SystemModuleEnum::Scheduler => "Scheduler",
            SystemModuleEnum::TaskArchive => "TaskArchive",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for SystemModuleEnum {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Dispatcher" => Ok(SystemModuleEnum::Dispatcher),
            "Harvester" => Ok(SystemModuleEnum::Harvester),
            "Hibernator" => Ok(SystemModuleEnum::Hibernator),
            "Receiver" => Ok(SystemModuleEnum::Receiver),
            "Scheduler" => Ok(SystemModuleEnum::Scheduler),
            "TaskArchive" => Ok(SystemModuleEnum::TaskArchive),
            _ => Err(()),
        }
    }
}

impl FromSql<Text, Pg> for SystemModuleEnum {
    fn from_sql(value: PgValue<'_>) -> diesel::deserialize::Result<Self> {
        match value.as_bytes() {
            b"Dispatcher" => Ok(SystemModuleEnum::Dispatcher),
            b"Harvester" => Ok(SystemModuleEnum::Harvester),
            b"Hibernator" => Ok(SystemModuleEnum::Hibernator),
            b"Receiver" => Ok(SystemModuleEnum::Receiver),
            b"Scheduler" => Ok(SystemModuleEnum::Scheduler),
            b"TaskArchive" => Ok(SystemModuleEnum::TaskArchive),
            _ => Err("Unexpected value".into()),
        }
    }
}

impl ToSql<Text, Pg> for SystemModuleEnum {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        match self {
            SystemModuleEnum::Dispatcher => out.write_all(b"Dispatcher")?,
            SystemModuleEnum::Harvester => out.write_all(b"Harvester")?,
            SystemModuleEnum::Hibernator => out.write_all(b"Hibernator")?,
            SystemModuleEnum::Receiver => out.write_all(b"Receiver")?,
            SystemModuleEnum::Scheduler => out.write_all(b"Scheduler")?,
            SystemModuleEnum::TaskArchive => out.write_all(b"TaskArchive")?,
        }
        Ok(diesel::serialize::IsNull::No)
    }
}
