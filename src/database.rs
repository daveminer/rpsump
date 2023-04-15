use rusqlite::{Connection, Result};
use std::io::{Error, ErrorKind};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(path: String) -> Result<Database, Error> {
        match Connection::open(path) {
            Ok(conn) => Ok(Database {
                conn: Arc::from(Mutex::new(conn)),
            }),
            Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
        }
    }
}
