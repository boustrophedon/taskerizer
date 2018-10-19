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

mod create;
mod backend;

pub use self::backend::DBBackend;

#[cfg(test)]
pub(crate) mod tests;
