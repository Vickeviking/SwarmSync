use diesel::deserialize::{FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::ToSql;
use diesel::sql_types::Text;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::Write;
use std::str::FromStr;

/// ScheduleTypeEnum, how is the job scheduled
#[derive(AsExpression, Debug, FromSqlRow, Serialize, Deserialize, Clone, PartialEq)]
#[diesel(sql_type = diesel::sql_types::VarChar)]
pub enum ScheduleTypeEnum {
    Once,
    Cron,
}

// serialize to json, and display
impl fmt::Display for ScheduleTypeEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            ScheduleTypeEnum::Once => "Once",
            ScheduleTypeEnum::Cron => "Cron",
        };
        write!(f, "{}", label)
    }
}

// deserialize from json
impl FromStr for ScheduleTypeEnum {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Once" => Ok(ScheduleTypeEnum::Once),
            "Cron" => Ok(ScheduleTypeEnum::Cron),
            _ => Err(()),
        }
    }
}

// deserialize from database
impl FromSql<Text, Pg> for ScheduleTypeEnum {
    fn from_sql(value: PgValue<'_>) -> diesel::deserialize::Result<Self> {
        match value.as_bytes() {
            b"Once" => Ok(ScheduleTypeEnum::Once),
            b"Cron" => Ok(ScheduleTypeEnum::Cron),
            _ => Err("Unexpected value for ScheduleTypeEnum".into()),
        }
    }
}

// serialize to database
impl ToSql<Text, Pg> for ScheduleTypeEnum {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        out.write_all(self.to_string().as_bytes())?;
        Ok(diesel::serialize::IsNull::No)
    }
}
