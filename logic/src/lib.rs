use sea_orm::{ConnectionTrait, Database, DbConn};
use tokio::runtime::Handle;

pub mod admins;
pub mod attendances;
pub mod attendees;
pub mod entity;
pub mod error;
pub mod instructors;
pub mod prelude;
pub mod subjects;

pub fn get_testing_db(base_url: &str, db_name: &str) -> DbConn {
    tokio::task::block_in_place(move || {
        Handle::current().block_on(async {
            let db = Database::connect(base_url).await.unwrap();

            db.execute_unprepared(&format!("DROP DATABASE IF EXISTS {db_name};"))
                .await
                .unwrap();
            db.execute_unprepared(&format!("CREATE DATABASE {db_name};"))
                .await
                .unwrap();

            let db = Database::connect(&format!("{base_url}/{db_name}"))
                .await
                .unwrap();

            db.execute_unprepared(include_str!("../../db-schema/init.sql"))
                .await
                .unwrap();

            db
        })
    })
}
