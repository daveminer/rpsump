use actix_web::web;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::sqlite::SqliteConnection;

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;
pub type DbConn = PooledConnection<ConnectionManager<SqliteConnection>>;

pub fn new_pool(path: &String) -> DbPool {
    let manager = ConnectionManager::<SqliteConnection>::new(path);

    Pool::builder()
        .build(manager)
        .expect("Could not create database pool.")
}

pub fn conn(db: web::Data<DbPool>) -> DbConn {
    db.get().expect("Could not acquire db lock.")
}
