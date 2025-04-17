use diesel::deserialize::{FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Text;
use serde::{Deserialize, Serialize};
use std::hash;
use std::io::Write;
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

impl ToString for SystemModuleEnum {
    fn to_string(&self) -> String {
        match self {
            SystemModuleEnum::Dispatcher => "Dispatcher".to_string(),
            SystemModuleEnum::Harvester => "Harvester".to_string(),
            SystemModuleEnum::Hibernator => "Hibernator".to_string(),
            SystemModuleEnum::Receiver => "Receiver".to_string(),
            SystemModuleEnum::Scheduler => "Scheduler".to_string(),
            SystemModuleEnum::TaskArchive => "TaskArchive".to_string(),
        }
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
