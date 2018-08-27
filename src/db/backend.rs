use failure::Error;
use db::{SqliteBackend, DBMetadata};

use task::Task;

pub trait DBBackend {
    type DBError;
    /// Get metadata about database
    fn metadata(&self) -> Result<DBMetadata, Self::DBError>;

    /// Add task to database
    fn add_task(&self, task: &Task) -> Result<(), Self::DBError>;

    /// Return a `Vec` of all tasks from the database
    fn get_all_tasks(&self) -> Result<Vec<Task>, Self::DBError>;

    /// Close the database. This is not really required due to the implementation of Drop for the
    /// Sqlite connection, but it might be necessary for other implementations e.g. a mock.
    fn close(self) -> Result<(), Self::DBError>;
}

// TODO currently we just format_err into essentially error strings because all we will do is
// display the error string to the user anyway, but it may be useful at some point (eg syncing over
// the network) to make a db error type so that we can distinguish them - eg retrying a network
// query later.

// TODO use get_checked instead of get when deserializing rows. i feel like there should be a
// functional way to do it so that we get a Result<T> at the end. in particular the error messages
// are bad - they should be something like format_err!("error trying to deserialize foo field from database
// row: {}", e) but there should be a way to do it without writing that out every time, instead
// just passing in the field or a tuple of idx and field name. there's a column_count on the row so
// you might not even need to pass in the idxes of field names, just the names.

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
        ).map_err(|e| format_err!("Error getting metadata from database: {}", e))?;
        Ok(
            DBMetadata {
                version: version,
                date_created: date_created,
            }
        )
    }

    fn add_task(&self, task: &Task) -> Result<(), Error> {
        self.connection.execute_named(
            "INSERT INTO tasks (task, priority, category) VALUES (:task, :priority, :category)",
            &[(":task", &task.task),
              (":priority", &task.priority),
              (":category", &task.reward)
            ],
        ).map_err(|e| format_err!("Error inserting task into database: {}", e))?;

        Ok(())
    }

    fn get_all_tasks(&self) -> Result<Vec<Task>, Self::DBError> {
        let mut tasks = Vec::new();

        let mut stmt = self.connection.prepare_cached(
            "SELECT task, priority, category
            FROM tasks
            ORDER BY
             category DESC,
             priority DESC,
             task ASC
            ")
            .map_err(|e| format_err!("Error preparing task list query: {}", e))?;
        let rows = stmt.query_map(&[], |row| {
                Task {
                    task: row.get(0),
                    priority: row.get(1),
                    reward: row.get(2)
                }
             })
            .map_err(|e| format_err!("Error executing task list query: {}", e))?;

        for task_res in rows {
            let task = task_res.map_err(|e| format_err!("Error deserializing task row from database: {}", e))?;
            tasks.push(task);
        }

        Ok(tasks)
    }

    fn close(self) -> Result<(), Error> {
        self.connection.close().map_err(|(_,e)| e.into())
    }
}
