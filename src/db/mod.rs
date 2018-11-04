use chrono::{DateTime, Utc};
use rusqlite::Connection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DBMetadata {
    pub version: String, // TODO use semver crate so we can compare minor patch versions etc.
    pub date_created: DateTime<Utc>,
}

#[derive(Debug)]
pub struct SqliteBackend {
    connection: Connection,
}

// pub struct SqliteTransaction<'conn> {
//     transaction: Transaction<'conn>,
// }
// 
// impl SqliteBackend {
//     fn transaction<'conn>(&'conn mut self) -> Result<SqliteTransaction, Error> {
//         let tx = self.connection.transaction()
//             .map_err(|e| format_err!("Could not begin sqlite transaction: {}", e))?;
// 
//         Ok(SqliteTransaction {
//             transaction: tx,
//         })
//     }
// }


mod create;
mod backend;

pub use self::backend::DBBackend;

#[cfg(test)]
pub(crate) mod tests;
