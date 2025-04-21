use diesel_async::{AsyncConnection, AsyncPgConnection};

pub mod models;
pub mod repositories;
pub mod schema;

//use me as handle to db
async fn load_db_connection() -> AsyncPgConnection {
    let database_url = std::env::var("DATABASE_URL").expect("Cannot load DB url from environment");
    AsyncPgConnection::establish(&database_url)
        .await
        .expect("Cannot connect to Postgres")
}
