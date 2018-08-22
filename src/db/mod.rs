use std::path::Path;

use failure::{Error, err_msg};
use rusqlite::{Connection, Transaction};
use chrono::{Utc, DateTime};

// TODO don't expose sqlitebackend type, make a open_sqlite(), open_sqlite_in_memory(),
// open_whatever() that all return an impl DBBackend

pub struct DBMetadata {
    pub version: String, // TODO use semver crate so we can compare minor patch versions etc.
    pub date_created: DateTime<Utc>,
}

pub trait DBBackend {
    type DBError;
    fn metadata(&self) -> Result<DBMetadata, Self::DBError>;
    fn close(self) -> Result<(), Self::DBError>;
}

pub struct SqliteBackend {
    connection: Connection,
}

// Open/create impls
impl SqliteBackend {
    /// Creates a taskerizer database at the given path if it does not exist, and opens and returns
    /// an existing one if there already was one. Path must be a directory and not a file.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<SqliteBackend, Error> {
        let mut path = path.as_ref().to_path_buf();
        
        if !path.is_dir() {
            return Err(err_msg("Database directory path is not a directory, or we do not have permission to access it."))
        }

        path.push("tkzr_sqlite3.db");

        let existing_db = path.is_file();

        let conn = Connection::open(path)?;
        let mut db = SqliteBackend {
            connection: conn,
        };

        // create db tables and populate metadata table
        if !existing_db {
            db.create_tables()?;
        }
        Ok(db)
    }

    pub fn open_in_memory() -> Result<SqliteBackend, Error> {
        Err(err_msg("not implemented"))
    }

}

// Create table impls
impl SqliteBackend {
    fn create_tables(&mut self) -> Result<(), Error> {
        //self.enable_foreign_keys_pragma()?;
        self.create_metadata_table()?;
        // self.create_tasks_table()?;
        // self.create_current_table()?;
        // self.create_completed_table()?;
        Ok(())
    }

    fn create_metadata_table(&mut self) -> Result<(), Error> {
        let conn = &self.connection;

        conn.execute(
            "CREATE TABLE metadata (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                version TEXT NOT NULL,
                date_created TEXT NOT NULL
            )",
            &[]
        ).map_err(|e| format_err!("Could not create metadata table: {}", e))?;

        let date_created = Utc::now().to_rfc3339();
        let version = env!("CARGO_PKG_VERSION");
        conn.execute(
            "INSERT INTO metadata (id, version, date_created) VALUES (
                1,
                ?1,
                ?2
            )",
            &[&version, &date_created]
        ).map_err(|e| format_err!("Could not insert metadata into database: {}", e))?;

        Ok(())
    }
}


impl DBBackend for SqliteBackend {
    type DBError = Error;

    fn metadata(&self) -> Result<DBMetadata, Error> {
        let (version, date_created) = self.connection.query_row(
            "SELECT version, date_created FROM metadata WHERE id = 1",
            &[],
            |row| {
                let version = row.get(0);
                let date_created = row.get(1);
                (version, date_created)
            }
        )?;
        Ok(
            DBMetadata {
                version: version,
                date_created: date_created,
            }
        )
    }

    fn close(self) -> Result<(), Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
