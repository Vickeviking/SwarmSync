use diesel::deserialize::{FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::ToSql;
use diesel::sql_types::Text;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::str::FromStr;

#[derive(AsExpression, Debug, Deserialize, Serialize, FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::VarChar)]
pub enum LogLevelEnum {
    Info,    //Expire in 5 minutes
    Success, //Expire in 1 day
    Warning, //Expire in 3 days
    Error,   // Expire in 7 days
    Fatal,
}

impl ToString for LogLevelEnum {
    fn to_string(&self) -> String {
        match self {
            LogLevelEnum::Info => String::from("Info"),
            LogLevelEnum::Success => String::from("Success"),
            LogLevelEnum::Warning => String::from("Warning"),
            LogLevelEnum::Error => String::from("Error"),
            LogLevelEnum::Fatal => String::from("Fatal"),
        }
    }
}

impl FromStr for LogLevelEnum {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Info" => Ok(LogLevelEnum::Info),
            "Success" => Ok(LogLevelEnum::Success),
            "Warning" => Ok(LogLevelEnum::Warning),
            "Error" => Ok(LogLevelEnum::Error),
            "Fatal" => Ok(LogLevelEnum::Fatal),
            _ => Err(()),
        }
    }
}

impl FromSql<Text, Pg> for LogLevelEnum {
    fn from_sql(value: PgValue<'_>) -> diesel::deserialize::Result<Self> {
        match value.as_bytes() {
            b"Info" => Ok(LogLevelEnum::Info),
            b"Success" => Ok(LogLevelEnum::Success),
            b"Warning" => Ok(LogLevelEnum::Warning),
            b"Error" => Ok(LogLevelEnum::Error),
            b"Fatal" => Ok(LogLevelEnum::Fatal),
            _ => Err("Unexpected value".into()),
        }
    }
}

impl ToSql<Text, Pg> for LogLevelEnum {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        match self {
            LogLevelEnum::Info => out.write_all(b"Info")?,
            LogLevelEnum::Success => out.write_all(b"Success")?,
            LogLevelEnum::Warning => out.write_all(b"Warning")?,
            LogLevelEnum::Error => out.write_all(b"Error")?,
            LogLevelEnum::Fatal => out.write_all(b"Fatal")?,
        }
        Ok(diesel::serialize::IsNull::No)
    }
}

#[derive(AsExpression, Debug, Deserialize, Serialize, FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::VarChar)]
pub enum LogActionEnum {
    ClientConnected,
    JobSubmitted,
    JobCompleted,
    SystemStarted,
    SystemShutdown,
    Custom,
}

impl ToString for LogActionEnum {
    fn to_string(&self) -> String {
        match self {
            LogActionEnum::ClientConnected => String::from("ClientConnected"),
            LogActionEnum::JobSubmitted => String::from("JobSubmitted"),
            LogActionEnum::JobCompleted => String::from("JobCompleted"),
            LogActionEnum::SystemStarted => String::from("SystemStarted"),
            LogActionEnum::SystemShutdown => String::from("SystemShutdown"),
            LogActionEnum::Custom => String::from("Custom"),
        }
    }
}

impl FromStr for LogActionEnum {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ClientConnected" => Ok(LogActionEnum::ClientConnected),
            "JobSubmitted" => Ok(LogActionEnum::JobSubmitted),
            "JobCompleted" => Ok(LogActionEnum::JobCompleted),
            "SystemStarted" => Ok(LogActionEnum::SystemStarted),
            "SystemShutdown" => Ok(LogActionEnum::SystemShutdown),
            "Custom" => Ok(LogActionEnum::Custom),
            _ => Err(()),
        }
    }
}

impl FromSql<Text, Pg> for LogActionEnum {
    fn from_sql(value: PgValue<'_>) -> diesel::deserialize::Result<Self> {
        match value.as_bytes() {
            b"ClientConnected" => Ok(LogActionEnum::ClientConnected),
            b"JobSubmitted" => Ok(LogActionEnum::JobSubmitted),
            b"JobCompleted" => Ok(LogActionEnum::JobCompleted),
            b"SystemStarted" => Ok(LogActionEnum::SystemStarted),
            b"SystemShutdown" => Ok(LogActionEnum::SystemShutdown),
            b"Custom" => Ok(LogActionEnum::Custom),
            _ => Err("Unexpected value".into()),
        }
    }
}

impl ToSql<Text, Pg> for LogActionEnum {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        match self {
            LogActionEnum::ClientConnected => out.write_all(b"ClientConnected")?,
            LogActionEnum::JobSubmitted => out.write_all(b"JobSubmitted")?,
            LogActionEnum::JobCompleted => out.write_all(b"JobCompleted")?,
            LogActionEnum::SystemStarted => out.write_all(b"SystemStarted")?,
            LogActionEnum::SystemShutdown => out.write_all(b"SystemShutdown")?,
            LogActionEnum::Custom => out.write_all(b"Custom")?,
        }
        Ok(diesel::serialize::IsNull::No)
    }
}
