use diesel::{Connection, MysqlConnection};
use dotenv::dotenv;
use std::env;

pub mod models;
pub mod schema;

pub fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}
