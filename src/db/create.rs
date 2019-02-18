use std::path::Path;

use chrono::Utc;
use rusqlite::{Connection, NO_PARAMS};
use failure::Error;

use crate::db::SqliteBackend;

// Open impls
impl SqliteBackend {
    /// Creates a taskerizer database at the given path if it does not exist, and opens and returns
    /// an existing one if there already was one. Path must be a directory and not a file.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<SqliteBackend, Error> {
        let mut path = path.as_ref().to_path_buf();
        
        if !path.is_dir() {
            return Err(format_err!("Database directory path \"{}\" is not a directory, or we do not have permission to access it.",
                                   path.to_string_lossy()));
        }

        path.push("tkzr_sqlite3.db");

        let existing_db = path.is_file();

        let conn = Connection::open(path)?;
        let db = SqliteBackend {
            connection: conn,
        };

        // create db tables and populate metadata table
        if !existing_db {
            db.create_tables()?;
        }
        Ok(db)
    }

    #[cfg(test)]
    /// Creates a taskerizer database in-memory. Only exists for testing purposes.
    // There is a small amount of duplication here (eg the db.create_tables() call) but it's
    // probably fine, though it could be the source of bugs if I change code in open but not in
    // here.
    pub fn open_in_memory() -> Result<SqliteBackend, Error> {
        let conn = Connection::open_in_memory()?;
        let db = SqliteBackend {
            connection: conn,
        };

        db.create_tables()?;
        Ok(db)
    }
}

// Create table impls
impl SqliteBackend {
    fn create_tables(&self) -> Result<(), Error> {
        self.enable_foreign_keys_pragma()?;
        self.create_metadata_table()?;
        self.create_tasks_table()?;
        self.create_current_table()?;
        self.create_replicas_table()?;
        self.create_servers_table()?;
        self.create_unsynced_ops_table()?;
        //self.create_completed_table()?;
        Ok(())
    }

    fn enable_foreign_keys_pragma(&self) -> Result<(), Error> {
        let conn = &self.connection;

        conn.execute(
            "PRAGMA foreign_keys = ON;",
            NO_PARAMS,
        ).map_err(|e| format_err!("Could not enable foreign keys pragma: {}", e))?;

        Ok(())
    }

    fn create_metadata_table(&self) -> Result<(), Error> {
        let conn = &self.connection;

        conn.execute(
            "CREATE TABLE metadata (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                version TEXT NOT NULL,
                date_created TEXT NOT NULL
            )",
            NO_PARAMS,
        ).map_err(|e| format_err!("Could not create metadata table: {}", e))?;

        let date_created = Utc::now().to_rfc3339();
        let version = env!("CARGO_PKG_VERSION");
        conn.execute(
            "INSERT INTO metadata (id, version, date_created) VALUES (
                1,
                ?1,
                ?2
            )",
            &[&version, date_created.as_str()]
        ).map_err(|e| format_err!("Could not insert metadata into database: {}", e))?;

        Ok(())
    }

    fn create_tasks_table(&self) -> Result<(), Error> {
        let conn = &self.connection;

        conn.execute(
            "CREATE TABLE tasks (
                id INTEGER PRIMARY KEY,
                task TEXT NOT NULL,
                priority INTEGER NOT NULL,
                category INTEGER NOT NULL,
                uuid BLOB UNIQUE NOT NULL
            );",
            NO_PARAMS,
        ).map_err(|e| format_err!("Could not create tasks table: {}", e))?;

        Ok(())
    }

    fn create_current_table(&self) -> Result<(), Error> {
        let conn = &self.connection;

        conn.execute(
            "CREATE TABLE current (
                id INTEGER PRIMARY KEY check (id = 1),
                task_id INTEGER NOT NULL,
                FOREIGN KEY (task_id) REFERENCES tasks(id)
            );",
            NO_PARAMS,
        ).map_err(|e| format_err!("Could not create current task table: {}", e))?;

        Ok(())
    }

    /// Create the `servers` table in the database. `api_url` is the url of the HTTP API being
    /// served at that URL.
    fn create_servers_table(&self) -> Result<(), Error> {
        let conn = &self.connection;

        conn.execute(
            "CREATE TABLE servers (
                id INTEGER PRIMARY KEY,
                api_url TEXT UNIQUE NOT NULL,
                replica_id INTEGER UNIQUE NOT NULL,
                FOREIGN KEY (replica_id) REFERENCES replicas(id)
            );",
            NO_PARAMS,
        ).map_err(|e| format_err!("Could not create servers table: {}", e))?;

        Ok(())
    }

    /// Create the `replicas` table in the database.
    fn create_replicas_table(&self) -> Result<(), Error> {
        let conn = &self.connection;

        conn.execute(
            "CREATE TABLE replicas (
                id INTEGER PRIMARY KEY,
                replica_uuid BLOB UNIQUE NOT NULL
            );",
            NO_PARAMS,
        ).map_err(|e| format_err!("Could not create replicas table: {}", e))?;

        Ok(())
    }

    fn create_unsynced_ops_table(&self) -> Result<(), Error> {
        let conn = &self.connection;

        // NOTE: text, priority, and category fields may be null. if any of them are null, all
        // three must be null and is_add_operation must be false.
        //
        // NOTE 2: sqlite's INTEGER PRIMARY KEY/rowid is monotonically increasing, so as long as we
        // don't exceed max i64 number of unsynced ops, storing the unsynced ops in order will
        // preserve the order.
        //
        // NOTE 3: unlike tasks table, task_uuid can't be UNIQUE because we support multiple
        // remove operations queued: e.g. we distribute a task to 3 clients, then two of them send
        // removes to the server. When the third syncs there will be two removes queued (and that's
        // okay because of how the U-Set CRDT works).
        //
        // FIXME: Add messages technically should be unique on (task_uuid, client_uuid) - but this
        // will basically never happen because adds are produced only by one client. (it could
        // happen due to a bug, like clients retransmitting recevied messages, which is why this is
        // a FIXME)
        //
        // Technically, we could store the sender as well as the recipient and check UNIQUE on the
        // pair, but it's not really worth it.
        //
        // In the future, we may want to manually deduplicate the messages, and in that case we
        // could add a UNIQUE constraint on the tuple (is_add_operation, task_uuid, client_uuid).
        conn.execute(
            "CREATE TABLE unsynced_ops (
                id INTEGER PRIMARY KEY,
                is_add_operation INTEGER,
                task TEXT,
                priority INTEGER,
                category INTEGER,
                task_uuid BLOB NOT NULL,
                client_uuid BLOB NOT NULL
            );",
            NO_PARAMS,
        ).map_err(|e| format_err!("Could not create unsynced ops table: {}", e))?;

        Ok(())
    }

    //fn create_completed_table(&self) -> Result<(), Error> {
    //    let conn = &self.connection;
    //    conn.execute(
    //        "CREATE TABLE completed (
    //            id INTEGER PRIMARY KEY,
    //            task TEXT NOT NULL,
    //            priority INTEGER NOT NULL,
    //            category INTEGER NOT NULL,
    //            date_completed TEXT NOT NULL
    //        );",
    //        NO_PARAMS,
    //    ).map_err(|e| format_err!("Could not create completed task table: {}", e))?;

    //    Ok(())
    //}
}

