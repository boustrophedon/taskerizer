use failure::Error;
use crate::db::DBMetadata;
use crate::db::{SqliteTransaction, DBTransaction};

use crate::selection::SelectionStrategy;

use crate::task::{Category, Task};

use rusqlite::NO_PARAMS;

pub trait DBBackend {
    /// Get metadata about database
    fn metadata(&self) -> Result<DBMetadata, Error>;

    /// Add task to database
    fn add_task(&self, task: &Task) -> Result<(), Error>;

    /// Return a `Vec` of all tasks from the database
    fn fetch_all_tasks(&self) -> Result<Vec<Task>, Error>;

    /// Returns the currently selected task if there is one, or None if there are no tasks in the
    /// database.  This function should never return None if there are tasks in the database.
    fn fetch_current_task(&self) -> Result<Option<Task>, Error>;

    /// Select a new current task according to the `SelectionStrategy` passed in as `selector`. If
    /// there are no tasks of one type, it will use the other. If there are both, the
    /// `SelectionStrategy` will choose one. If there are no tasks in the database, do nothing.
    fn select_current_task(&self, selector: &mut dyn SelectionStrategy) -> Result<(), Error>;

    /// Replace the current task with a new one, leaving the previous current task in the database.
    //fn skip_current_task(&self, selector: &mut dyn SelectionStrategy) -> Result<(), Error>;

    /// Replace the current task with a new one, removing the previous current task from the
    /// database and marking it as completed.
    fn complete_current_task(&self, selector: &mut dyn SelectionStrategy) -> Result<Option<Task>, Error>;

    /// Finish database operations, committing to the database. If this is not called, the
    /// transaction is rolled back.
    fn finish(self) -> Result<(), Error>;

}

// TODO currently we just format_err into essentially error strings because all we will do is
// display the error string to the user anyway, but it may be useful at some point (eg syncing over
// the network) to make a db error type so that we can distinguish them - eg retrying a network
// query later.

// TODO use get_checked() instead of get() when deserializing rows. i feel like there should be a
// functional way to do it so that we get a Result<T> at the end. in particular the error messages
// are bad - they should be something like format_err!("error trying to deserialize foo field from database
// row: {}", e) but there should be a way to do it without writing that out every time, instead
// just passing in the field or a tuple of idx and field name. there's a column_count on the row so
// you might not even need to pass in the idxes of field names, just the names.

impl<'conn> DBBackend for SqliteTransaction<'conn> {
    fn metadata(&self) -> Result<DBMetadata, Error> {
        let tx = &self.transaction;

        let (version, date_created) = tx.query_row(
            "SELECT version, date_created FROM metadata WHERE id = 1",
            NO_PARAMS,
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
        DBTransaction::add_task(self, task) 
    }

    fn fetch_all_tasks(&self) -> Result<Vec<Task>, Error> {
        let tx = self;
        let tasks = tx.fetch_tasks()
            .map_err(|e| format_err!("Failed to get tasks during transaction: {}", e))?;
        let breaks = tx.fetch_breaks()
            .map_err(|e| format_err!("Failed to get break tasks during transaction: {}", e))?;

        // chain tasks followed by breaks into single vector
        let all_tasks = tasks.into_iter().map(|t| t.1)
            .chain(breaks.into_iter().map(|t| t.1))
            .collect();

        return Ok(all_tasks);
    }

    fn fetch_current_task(&self) -> Result<Option<Task>, Error> {
        DBTransaction::fetch_current_task(self) 
    }

    fn select_current_task(&self, selector: &mut dyn SelectionStrategy) -> Result<(), Error> {
        let tx = self;

        let tasks = tx.fetch_tasks()
            .map_err(|e| format_err!("Failed to get tasks during transaction: {}", e))?;

        let breaks = tx.fetch_breaks()
            .map_err(|e| format_err!("Failed to get break tasks during transaction: {}", e))?;

        // If there are no tasks or no breaks, must select the other unless there are none
        // If there are some of both, use the selection strategy
        let selected_tasks = match (tasks.len(), breaks.len()) {
            // None of either => no tasks in db, so there cannot be a current task.
            (0, 0) => return Ok(()),
            // no tasks, use breaks
            (0, _) => breaks,
            (_, 0) => tasks,
            (_, _) => {
                let category = selector.select_category();
                match category {
                    Category::Break => {
                        breaks
                    },
                    Category::Task => {
                        tasks
                    }
                }
            }
        };

        let tasks_refs: Vec<&Task> = selected_tasks.iter().map(|t| &t.1).collect();

        let selected_task_idx = selector.select_task(&tasks_refs);

        tx.set_current_task(&selected_tasks[selected_task_idx].0)
            .map_err(|e| format_err!("Failed to set current task during transaction: {}", e))?;


        Ok(())
    }

    ///// Replace the current task with a new one, leaving the previous current task in the database.
    //fn skip_current_task(&self, selector: &mut dyn SelectionStrategy) -> Result<(), Error>;

    /// Replace the current task with a new one, removing the previous current task from the
    /// database and returning it.
    fn complete_current_task(&self, selector: &mut dyn SelectionStrategy) -> Result<Option<Task>, Error> {
        let tx = self;

        let current_opt = tx.pop_current_task()
            .map_err(|e| format_err!("Failed to pop current task during transaction: {}", e))?;
        let (current_task_id, current_task) = match current_opt {
            Some((id, task)) => (id, task),
            None => return Ok(None),
        };

        tx.remove_task(&current_task_id)
            .map_err(|e| format_err!("Failed to remove task during transaction: {}", e))?;

        tx.select_current_task(selector)
            .map_err(|e| format_err!("Failed to select new current task during transaction: {}", e))?;

        Ok(Some(current_task))
    }

    fn finish(self) -> Result<(), Error> {
        self.commit()
    }

}
