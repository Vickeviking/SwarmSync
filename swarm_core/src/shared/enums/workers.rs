use diesel::deserialize::{FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::ToSql;
use diesel::sql_types::Text;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::Write;
use std::str::FromStr;

#[derive(AsExpression, Debug, FromSqlRow, Serialize, Deserialize, Clone, PartialEq)]
#[diesel(sql_type = diesel::sql_types::VarChar)]
pub enum WorkerStatusEnum {
    Idle,
    Busy,
    Offline,
    Unreachable,
}

impl fmt::Display for WorkerStatusEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            WorkerStatusEnum::Idle => "Idle",
            WorkerStatusEnum::Busy => "Busy",
            WorkerStatusEnum::Offline => "Offline",
            WorkerStatusEnum::Unreachable => "Unreachable",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for WorkerStatusEnum {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Idle" => Ok(WorkerStatusEnum::Idle),
            "Busy" => Ok(WorkerStatusEnum::Busy),
            "Offline" => Ok(WorkerStatusEnum::Offline),
            "Unreachable" => Ok(WorkerStatusEnum::Unreachable),
            _ => Err(()),
        }
    }
}

impl FromSql<Text, Pg> for WorkerStatusEnum {
    fn from_sql(value: PgValue<'_>) -> diesel::deserialize::Result<Self> {
        match value.as_bytes() {
            b"Idle" => Ok(WorkerStatusEnum::Idle),
            b"Busy" => Ok(WorkerStatusEnum::Busy),
            b"Offline" => Ok(WorkerStatusEnum::Offline),
            b"Unreachable" => Ok(WorkerStatusEnum::Unreachable),
            _ => Err("Unexpected value".into()),
        }
    }
}

impl ToSql<Text, Pg> for WorkerStatusEnum {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        match self {
            WorkerStatusEnum::Idle => out.write_all(b"Idle")?,
            WorkerStatusEnum::Busy => out.write_all(b"Busy")?,
            WorkerStatusEnum::Offline => out.write_all(b"Offline")?,
            WorkerStatusEnum::Unreachable => out.write_all(b"Unreachable")?,
        }
        Ok(diesel::serialize::IsNull::No)
    }
}

#[derive(AsExpression, Debug, FromSqlRow, Serialize, Deserialize, Clone)]
#[diesel(sql_type = diesel::sql_types::VarChar)]
pub enum OSEnum {
    Linux,
    Windows,
    MacOSEnum,
    Any,
}

impl fmt::Display for OSEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            OSEnum::Linux => "Linux",
            OSEnum::Windows => "Windows",
            OSEnum::MacOSEnum => "MacOS",
            OSEnum::Any => "Any",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for OSEnum {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Linux" => Ok(OSEnum::Linux),
            "Windows" => Ok(OSEnum::Windows),
            "MacOSEnum" => Ok(OSEnum::MacOSEnum),
            "Any" => Ok(OSEnum::Any),
            _ => Err(()),
        }
    }
}

impl FromSql<Text, Pg> for OSEnum {
    fn from_sql(value: PgValue<'_>) -> diesel::deserialize::Result<Self> {
        match value.as_bytes() {
            b"Linux" => Ok(OSEnum::Linux),
            b"Windows" => Ok(OSEnum::Windows),
            b"MacOSEnum" => Ok(OSEnum::MacOSEnum),
            b"Any" => Ok(OSEnum::Any),
            _ => Err("Unexpected value".into()),
        }
    }
}

impl ToSql<Text, Pg> for OSEnum {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        match self {
            OSEnum::Linux => out.write_all(b"Linux")?,
            OSEnum::Windows => out.write_all(b"Windows")?,
            OSEnum::MacOSEnum => out.write_all(b"MacOSEnum")?,
            OSEnum::Any => out.write_all(b"Any")?,
        }
        Ok(diesel::serialize::IsNull::No)
    }
}
