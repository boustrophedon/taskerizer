use std::marker::PhantomData;

use failure::Error;
use crate::db::SqliteTransaction;

use crate::task::Task;

#[derive(Debug)]
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
             priority DESC
            ")
            .map_err(|e| format_err!("Error preparing task list query: {}", e))?;
        let rows = stmt.query_map(&[], |row| {
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
             priority DESC
            ")
            .map_err(|e| format_err!("Error preparing task list query: {}", e))?;
        let rows = stmt.query_map(&[], |row| {
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
        tx.execute_named(
            "REPLACE INTO current (id, task_id)
            VALUES (1, :task_id)",
            &[(":task_id", &id.id)])
            .map_err(|e| format_err!("Error updating current task in database: {}", e))?;
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

        let rows: Vec<_> = stmt.query_map(&[], |row| {
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
