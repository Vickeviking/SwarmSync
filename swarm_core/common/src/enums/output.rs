use std::fmt;
use std::io::Write;
use std::str::FromStr;

use diesel::deserialize::{FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::ToSql;
use diesel::sql_types::Text;
use serde::{Deserialize, Serialize};

/// Output Type, either stdout or files, payload provided as option in further fields inside job
#[derive(AsExpression, Debug, FromSqlRow, Serialize, Deserialize, PartialEq)]
#[diesel(sql_type = diesel::sql_types::VarChar)]
pub enum OutputTypeEnum {
    Stdout,
    Files, // Files will be stored separately
}

// serialize to json, and log as string
impl fmt::Display for OutputTypeEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            OutputTypeEnum::Stdout => "Stdout",
            OutputTypeEnum::Files => "Files",
        };
        write!(f, "{}", label)
    }
}

// deserialize from json
impl FromStr for OutputTypeEnum {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Stdout" => Ok(OutputTypeEnum::Stdout),
            "Files" => Ok(OutputTypeEnum::Files),
            _ => Err(()),
        }
    }
}

// deserialize from database
impl FromSql<Text, Pg> for OutputTypeEnum {
    fn from_sql(value: PgValue<'_>) -> diesel::deserialize::Result<Self> {
        match value.as_bytes() {
            b"Stdout" => Ok(OutputTypeEnum::Stdout),
            b"Files" => Ok(OutputTypeEnum::Files),
            _ => Err("Unexpected value".into()),
        }
    }
}

// serialize to database
impl ToSql<Text, Pg> for OutputTypeEnum {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        match self {
            OutputTypeEnum::Stdout => out.write_all(b"Stdout")?,
            OutputTypeEnum::Files => out.write_all(b"Files")?,
        }
        Ok(diesel::serialize::IsNull::No)
    }
}
