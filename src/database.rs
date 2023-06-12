use actix_web::web;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::sqlite::SqliteConnection;

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;
pub type DbConn = PooledConnection<ConnectionManager<SqliteConnection>>;

pub fn new_pool(path: &String) -> Result<DbPool, anyhow::Error> {
    let manager = ConnectionManager::<SqliteConnection>::new(path);

    Pool::builder().build(manager).map_err(|e| e.into())
}

pub fn conn(db: web::Data<DbPool>) -> Result<DbConn, anyhow::Error> {
    db.get().map_err(|e| e.into())
}
