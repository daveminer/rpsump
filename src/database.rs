use actix_web::web;
use anyhow::Error;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::sqlite::SqliteConnection;

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;
pub type DbConn = PooledConnection<ConnectionManager<SqliteConnection>>;

pub fn new_pool(path: &String) -> Result<DbPool, Error> {
    let manager = ConnectionManager::<SqliteConnection>::new(path);
    let pool = Pool::builder().build(manager)?;

    Ok(pool)
}

pub fn conn(db: web::Data<DbPool>) -> DbConn {
    db.get().expect("Could not acquire db lock.")
}

#[macro_export]
macro_rules! first {
    ($query:expr, $typ: ty, $db:expr) => {
        web::block(move || {
            let mut conn = $db.get().expect("Could not get db connection.");

            $query.first(&mut conn) as Result<$typ, diesel::result::Error>
        })
        .await?
    };
}

pub(crate) use first;
