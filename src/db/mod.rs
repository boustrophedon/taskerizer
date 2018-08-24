use std::path::Path;

use chrono::{DateTime, Utc};
use failure::Error;
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

pub fn make_sqlite_backend<P: AsRef<Path>>(path: P) -> Result<impl DBBackend, Error> {
    SqliteBackend::open(path)
}

mod create;
mod backend;

pub use self::backend::DBBackend;

#[cfg(test)]
mod tests;
