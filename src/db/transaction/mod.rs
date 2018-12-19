use std::marker::PhantomData;

use failure::Error;
use crate::db::SqliteTransaction;

use crate::task::Task;

use rusqlite::NO_PARAMS;

// FIXME: i'm kind of wary of allowing PartialEq and Eq to be derived for RowId because ideally
// they shouldn't need to be compared, but I did it to make the tests simpler. 
// Since their lifetime is tied to a transaction, ideally it shouldn't be a problem (and in fact
// it would be nice if a db guaranteed that rowids aren't reused within a transaction) but
// technically I guess something could go wrong.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowId<'tx, 'conn: 'tx> {
    id: i32,
    _transaction: PhantomData<&'tx SqliteTransaction<'conn>>
}

pub trait DBTransaction {
    /// Add task to database
    fn add_task(&self, task: &Task) -> Result<(), Error>;

    /// Return a `Vec` of all tasks (non-breaks) from the database. 
    fn get_tasks(&self) -> Result<Vec<(RowId, Task)>, Error>;

    /// Return a `Vec` of all breaks from the database.
    fn get_breaks(&self) -> Result<Vec<(RowId, Task)>, Error>;

    /// Set the current task to be the task with id `id`.
    fn set_current_task(&self, id: &RowId) -> Result<(), Error>;

    /// Returns the currently selected task if there is one, or None if there are no tasks in the
    /// database. This function should never return None if there are tasks in the database.
    fn get_current_task(&self) -> Result<Option<Task>, Error>;

    /// Returns the `RowId` of the currently selected task if there is one, or None if there are no
    /// tasks in the database. The current task is then set to None, but the task itself is not
    /// removed from the database. This function should never return None if there are tasks in the
    /// database.
    ///
    /// # Invariants
    /// The current task is set to None, so it must be set to something else via
    /// `DBTransaction::set_current_task` by the end of the transaction if there are other tasks in
    /// the database.
    fn pop_current_task(&self) -> Result<Option<RowId>, Error>;

    /// Remove the given task from the tasks table of the database. This operation will produce an
    /// error if the task is set as the current task and it is removed.
    fn remove_task(&self, id: &RowId) -> Result<(), Error>;

    /// Commit the transaction. If this method is not called, implementors of this trait should
    /// default to rolling back the transaction upon drop.
    fn commit(self) -> Result<(), Error>;

    /// Roll back the transaction. Implementors of this trait should default to rolling back the
    /// transaction upon drop.
    fn rollback(self) -> Result<(), Error>;
}

impl<'conn> DBTransaction for SqliteTransaction<'conn> {
    fn add_task(&self, task: &Task) -> Result<(), Error> {
        let tx = &self.transaction;
        tx.execute_named(
            "INSERT INTO tasks (task, priority, category) VALUES (:task, :priority, :category)",
            &[(":task", &task.task()),
              (":priority", &task.priority()),
              (":category", &task.is_break())
            ],
        ).map_err(|e| format_err!("Error inserting task into database: {}", e))?;
        Ok(())
    }

    fn get_tasks(&self) -> Result<Vec<(RowId, Task)>, Error> {
        let tx = &self.transaction;

        let mut tasks = Vec::new();
        let mut stmt = tx.prepare_cached(
            "SELECT id, task, priority, category
            FROM tasks
            WHERE category = 0
            ORDER BY
             category ASC,
             priority ASC
            ")
            .map_err(|e| format_err!("Error preparing task list query: {}", e))?;
        let rows = stmt.query_map(NO_PARAMS, |row| {
                let rowid = RowId { id: row.get(0), _transaction: PhantomData };
                let task = Task::from_parts(row.get(1), row.get(2), row.get(3));
                (rowid, task)
             })
            .map_err(|e| format_err!("Error executing task list query: {}", e))?;

        for row_res in rows {
            let (id, task_res) = row_res.map_err(|e| format_err!("Error deserializing task row from database: {}", e))?;
            let task = task_res.map_err(|e| format_err!("Invalid task read from database row: {}", e))?;
            tasks.push((id, task));
        }
        Ok(tasks)
    }

    fn get_breaks(&self) -> Result<Vec<(RowId, Task)>, Error> {
        let tx = &self.transaction;

        let mut tasks = Vec::new();
        let mut stmt = tx.prepare_cached(
            "SELECT id, task, priority, category
            FROM tasks
            WHERE category = 1
            ORDER BY
             category ASC,
             priority ASC
            ")
            .map_err(|e| format_err!("Error preparing task list query: {}", e))?;
        let rows = stmt.query_map(NO_PARAMS, |row| {
                let rowid = RowId { id: row.get(0), _transaction: PhantomData };
                let task = Task::from_parts(row.get(1), row.get(2), row.get(3));
                (rowid, task)
             })
            .map_err(|e| format_err!("Error executing task list query: {}", e))?;

        for row_res in rows {
            let (id, task_res) = row_res.map_err(|e| format_err!("Error deserializing task row from database: {}", e))?;
            let task = task_res.map_err(|e| format_err!("Invalid task read from database row: {}", e))?;
            tasks.push((id, task));
        }
        Ok(tasks)
    }

    fn set_current_task(&self, id: &RowId) -> Result<(), Error> {
        let tx = &self.transaction;
        let rows_modified = tx.execute_named(
            "REPLACE INTO current (id, task_id)
            VALUES (1, :task_id)",
            &[(":task_id", &id.id)])
            .map_err(|e| format_err!("Error updating current task in database: {}", e))?;

        if rows_modified == 0 {
            return Err(format_err!("Error updating current task in database: No rows were modified."));
        }
        else if rows_modified > 1 {
            return Err(format_err!("Error updating current task in database: Too many rows were modified: {}.", rows_modified));
        }

        Ok(())

    }

    fn get_current_task(&self) -> Result<Option<Task>, Error> {
        let tx = &self.transaction;
        let mut stmt = tx.prepare_cached(
            "SELECT task, priority, category
            FROM tasks
            WHERE id = (
                SELECT task_id FROM current
                WHERE id = 1
            )
            ")
            .map_err(|e| format_err!("Error preparing current task query: {}", e))?;

        let rows: Vec<_> = stmt.query_map(NO_PARAMS, |row| {
                Task::from_parts(row.get(0), row.get(1), row.get(2))
             })
            .map_err(|e| format_err!("Error executing current task query: {}", e))?
            .collect();

        if rows.len() > 1 {
            return Err(format_err!("Multiple tasks selected in current task query. {} tasks, selected {:?}", rows.len(), rows))
        }

        for row_res in rows {
            let task_res = row_res.map_err(|e| format_err!("Error deserializing task row from database: {}", e))?;
            let task = task_res.map_err(|e| format_err!("Invalid task read from database row: {}", e))?;
            return Ok(Some(task));
        }
        // if there are no rows, return Ok(None)
        return Ok(None);
    }

    fn pop_current_task(&self) -> Result<Option<RowId>, Error> {
        let tx = &self.transaction;
        let mut stmt = tx.prepare_cached(
            "SELECT task_id FROM current
            WHERE id = 1
            ")
            .map_err(|e| format_err!("Error preparing pop current task query: {}", e))?;

        let rows: Vec<_> = stmt.query_map(NO_PARAMS, |row| {
                RowId { id: row.get(0), _transaction: PhantomData }
             })
            .map_err(|e| format_err!("Error executing pop current task query: {}", e))?
            .collect();

        // if there are no rows i.e. no current task, return None
        if rows.len() == 0 {
            return Ok(None);
        }
        if rows.len() > 1 {
            return Err(format_err!("Multiple tasks selected in pop current task query. {} tasks, selected {:?}", rows.len(), rows))
        }

        let current_rowid = rows.into_iter().next()
            .expect("No rows even though we checked there was one")
            .map_err(|e| format_err!("Error deserializing task row from database: {}", e))?;

        let rows_modified = tx.execute(
            "DELETE FROM current
            WHERE id = 1
            ", NO_PARAMS)
            .map_err(|e| format_err!("Error unsetting current task: {}", e))?;
        if rows_modified == 0 {
            return Err(format_err!("Error unsetting task: No rows were deleted."));
        }
        else if rows_modified > 1 {
            return Err(format_err!("Error unsetting task: More than one row was deleted: {}.", rows_modified));
        }

        return Ok(Some(current_rowid));
    }

    fn remove_task(&self, id: &RowId) -> Result<(), Error> {
        let tx = &self.transaction;
        let rows_modified = tx.execute_named(
            "DELETE FROM tasks
            WHERE
                id = :task_id",
            &[(":task_id", &id.id)])
            .map_err(|e| format_err!("Error deleting task: {}", e))?;
        if rows_modified == 0 {
            return Err(format_err!("Error deleting task: No rows were deleted."));
        }
        else if rows_modified > 1 {
            return Err(format_err!("Error deleting task: More than one row was deleted: {}.", rows_modified));
        }
        Ok(())
    }

    fn commit(self) -> Result<(), Error> {
        let tx = self.transaction;

        tx.commit()
            .map_err(|e| format_err!("Error committing transaction: {}", e))?;
        Ok(())
    }

    fn rollback(self) -> Result<(), Error> {
        return Ok(());
    }
}

#[cfg(test)]
mod tests;
