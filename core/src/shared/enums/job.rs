use std::io::Write;
use std::str::FromStr;

use diesel::deserialize::FromSql;
use diesel::deserialize::FromSqlRow;
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::ToSql;
use diesel::sql_types::Text;
use serde::{Deserialize, Serialize};

#[derive(AsExpression, Debug, Deserialize, Serialize, FromSqlRow, PartialEq)]
#[diesel(sql_type = Text)]
pub enum JobStateEnum {
    Queued,
    Running,
    Completed,
    Failed, // Can store dynamic error message
}

impl ToString for JobStateEnum {
    fn to_string(&self) -> String {
        match self {
            JobStateEnum::Queued => String::from("Queued"),
            JobStateEnum::Running => String::from("Running"),
            JobStateEnum::Completed => String::from("Completed"),
            JobStateEnum::Failed => String::from("Failed"),
        }
    }
}

impl FromStr for JobStateEnum {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Queued" => Ok(JobStateEnum::Queued),
            "Running" => Ok(JobStateEnum::Running),
            "Completed" => Ok(JobStateEnum::Completed),
            "Failed" => Ok(JobStateEnum::Failed),
            _ => Err(()),
        }
    }
}

impl FromSql<Text, Pg> for JobStateEnum {
    fn from_sql(value: PgValue<'_>) -> diesel::deserialize::Result<Self> {
        match value.as_bytes() {
            b"Queued" => Ok(JobStateEnum::Queued),
            b"Running" => Ok(JobStateEnum::Running),
            b"Completed" => Ok(JobStateEnum::Completed),
            b"Failed" => Ok(JobStateEnum::Failed),
            _ => Err("Unexpected value".into()),
        }
    }
}

impl ToSql<Text, Pg> for JobStateEnum {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        match self {
            JobStateEnum::Queued => out.write_all(b"Queued")?,
            JobStateEnum::Running => out.write_all(b"Running")?,
            JobStateEnum::Completed => out.write_all(b"Completed")?,
            JobStateEnum::Failed => out.write_all(b"Failed")?,
        }
        Ok(diesel::serialize::IsNull::No)
    }
}

#[derive(AsExpression, Debug, Deserialize, Serialize, FromSqlRow, PartialEq, Eq)]
#[diesel(sql_type = Text)]
pub enum JobScheduleEnum {
    Once,
    Cron,
}

impl ToString for JobScheduleEnum {
    fn to_string(&self) -> String {
        match self {
            JobScheduleEnum::Once => String::from("Once"),
            JobScheduleEnum::Cron => String::from("Cron"),
        }
    }
}

impl FromStr for JobScheduleEnum {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Once" => Ok(JobScheduleEnum::Once),
            "Cron" => Ok(JobScheduleEnum::Cron), // We can initialize with an empty string
            _ => Err(()),
        }
    }
}

impl FromSql<Text, Pg> for JobScheduleEnum {
    fn from_sql(value: PgValue<'_>) -> diesel::deserialize::Result<Self> {
        match value.as_bytes() {
            b"Once" => Ok(JobScheduleEnum::Once),
            b"Cron" => Ok(JobScheduleEnum::Cron),
            _ => Err("Unexpected value".into()),
        }
    }
}

impl ToSql<Text, Pg> for JobScheduleEnum {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        match self {
            JobScheduleEnum::Once => out.write_all(b"Once")?,
            JobScheduleEnum::Cron => out.write_all(b"Cron")?,
        }
        Ok(diesel::serialize::IsNull::No)
    }
}
