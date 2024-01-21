use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::sqlite::SqliteConnection;
use mockall::automock;

pub type RealDbPool = Pool<ConnectionManager<SqliteConnection>>;
pub type DbConn = PooledConnection<ConnectionManager<SqliteConnection>>;

#[automock]
pub trait DbPool: Send + Sync {
    fn get_conn(&self) -> Result<DbConn, anyhow::Error>;
}

impl DbPool for RealDbPool {
    fn get_conn(&self) -> Result<DbConn, anyhow::Error> {
        self.get().map_err(|e| e.into())
    }
}

impl Clone for MockDbPool {
    fn clone(&self) -> Self {
        Self::default()
    }
}
