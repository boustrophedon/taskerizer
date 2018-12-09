use std::marker::PhantomData;

use failure::Error;
use db::SqliteTransaction;

use task::Task;

#[derive(Debug)]
struct RowId<'tx, 'conn: 'tx> {
    id: usize,
    _transaction: PhantomData<&'tx SqliteTransaction<'conn>>
}

pub trait DBTransaction {
    /// Add task to database
    fn add_task(&self, task: &Task) -> Result<(), Error>;

    // /// Return a `Vec` of all tasks (non-breaks) from the database. 
    // fn get_tasks(&self) -> Result<Vec<(RowId, Task)>, Error>;

    // /// Return a `Vec` of all breaks from the database.
    // fn get_breaks(&self) -> Result<Vec<(RowId, Task)>, Error>;

    // /// Set the current task to be the task with id `id`.
    // fn set_current_task(&self, id: &RowId) -> Result<(), Error>;

    // /// Returns the currently selected task if there is one, or None if there are no tasks in the
    // /// database. This function should never return None if there are tasks in the database.
    // fn get_current_task(&self) -> Result<Option<Task>, Error>;

    /// Commit the transaction. If this method is not called, implementors of this trait should
    /// default to rolling back the transaction upon drop.
    fn commit(self) -> Result<(), Error>;

    /// Roll back the transaction. Implementors of this trait should default to rolling back the
    /// transaction upon drop.
    fn rollback(self) -> Result<(), Error>;
}

impl<'conn> DBTransaction for SqliteTransaction<'conn> {
    fn add_task(&self, task: &Task) -> Result<(), Error> {
        return Ok(());
    }

    fn commit(self) -> Result<(), Error> {
        return Ok(());
    }

    fn rollback(self) -> Result<(), Error> {
        return Ok(());
    }
}

#[cfg(test)]
mod tests;
