use std::fmt;
use std::io::Write;
use std::str::FromStr;

use diesel::deserialize::FromSql;
use diesel::deserialize::FromSqlRow;
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::ToSql;
use diesel::sql_types::Text;
use serde::{Deserialize, Serialize};

/// The format of the downloaded image, how ashuold we extract it
#[derive(AsExpression, FromSqlRow, Debug, Deserialize, Serialize, PartialEq)]
#[diesel(sql_type = diesel::sql_types::VarChar)]
pub enum ImageFormatEnum {
    Tarball,
    DockerRegistry,
}

// For logging aswell as serializing to json
impl fmt::Display for ImageFormatEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            ImageFormatEnum::Tarball => "Tarball",
            ImageFormatEnum::DockerRegistry => "DockerRegistry",
        };
        write!(f, "{}", label)
    }
}

// deserialize from json
impl FromStr for ImageFormatEnum {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Tarball" => Ok(ImageFormatEnum::Tarball),
            "DockerRegistry" => Ok(ImageFormatEnum::DockerRegistry),
            _ => Err(()),
        }
    }
}

// deserialize from database
impl FromSql<Text, Pg> for ImageFormatEnum {
    fn from_sql(value: PgValue<'_>) -> diesel::deserialize::Result<Self> {
        match value.as_bytes() {
            b"Tarball" => Ok(ImageFormatEnum::Tarball),
            b"DockerRegistry" => Ok(ImageFormatEnum::DockerRegistry),
            _ => Err("Unexpected value for ImageFormat".into()),
        }
    }
}

// serialize to database
impl ToSql<Text, Pg> for ImageFormatEnum {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        out.write_all(self.to_string().as_bytes())?;
        Ok(diesel::serialize::IsNull::No)
    }
}
