// want to keep RowId usage as pass-by-ref
#![allow(clippy::trivially_copy_pass_by_ref)]

use std::marker::PhantomData;

use failure::Error;
use rusqlite::NO_PARAMS;
use rusqlite::types::{FromSql, FromSqlResult, FromSqlError, ValueRef};
use rusqlite::Result as SQLResult;
use uuid::Uuid;

use crate::db::SqliteTransaction;

use crate::task::Task;

// TODO: rusqlite has a FromSql<i128> but not u128, whereas Uuid has From<u128> but not From<i128>.
// so add a FromSql<u128> to rusqlite.
pub struct SqlBlobUuid {
    pub uuid: Uuid,
}

impl FromSql for SqlBlobUuid {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        use byteorder::{LittleEndian, ByteOrder};

        value.as_blob().and_then(|bytes| {
            if bytes.len() == 16 {
                let uuid = From::from(LittleEndian::read_u128(bytes));
                Ok(SqlBlobUuid { uuid } )
            } else {
                Err(FromSqlError::Other(format_err!("Invalid number of bytes for UUID: {} bytes != 16.", bytes.len()).into()))
            }
        })
    }
}

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
    /// Return a `Vec` of all tasks (non-breaks) from the database. 
    fn fetch_tasks(&self) -> Result<Vec<(RowId, Task)>, Error>;

    /// Return a `Vec` of all breaks from the database.
    fn fetch_breaks(&self) -> Result<Vec<(RowId, Task)>, Error>;

    /// Set the current task to be the task with id `id`.
    fn set_current_task(&self, id: &RowId) -> Result<(), Error>;

    /// Returns the `RowId` of the currently selected task if there is one, or None if there are no
    /// tasks in the database. The current task is then set to None, but the task itself is not
    /// removed from the database. This function should never return None if there are tasks in the
    /// database.
    ///
    /// # Invariants
    /// The current task is set to None, so it must be set to something else via
    /// `DBTransaction::set_current_task` by the end of the transaction if there are other tasks in
    /// the database.
    fn pop_current_task(&self) -> Result<Option<(RowId, Task)>, Error>;

    /// Remove the given task from the tasks table of the database. This operation will produce an
    /// error if the task is set as the current task and it is removed.
    fn remove_task(&self, id: &RowId) -> Result<(), Error>;

    /// Remove the task with given UUID from the tasks table of the database. This operation will
    /// produce an error if the task is set as the current task and it is removed.
    ///
    /// If there is no task with the corresponding UUID in the database, nothing happens.
    fn try_remove_task_by_uuid(&self, uuid: &Uuid) -> Result<(), Error>;

    /// Commit the transaction. If this method is not called, implementors of this trait should
    /// default to rolling back the transaction upon drop.
    fn commit(self) -> Result<(), Error>;

    /// Roll back the transaction. Implementors of this trait should default to rolling back the
    /// transaction upon drop.
    fn rollback(self) -> Result<(), Error>;
}

impl<'conn> DBTransaction for SqliteTransaction<'conn> {
    fn fetch_tasks(&self) -> Result<Vec<(RowId, Task)>, Error> {
        let tx = &self.transaction;

        let mut tasks = Vec::new();
        let mut stmt = tx.prepare_cached(
            "SELECT id, task, priority, category, uuid
            FROM tasks
            WHERE category = 0
            ORDER BY
             category ASC,
             priority ASC
            ")
            .map_err(|e| format_err!("Error preparing task list query: {}", e))?;
        let rows = stmt.query_map(NO_PARAMS, |row| {
                let row_id = RowId { id: row.get(0)?, _transaction: PhantomData };
                let task_text = row.get(1)?;
                let task_priority = row.get(2)?;
                let task_reward = row.get(3)?;
                let sql_uuid: SqlBlobUuid = row.get(4)?;

                Ok((row_id, task_text, task_priority, task_reward, sql_uuid.uuid))
             })
            .map_err(|e| format_err!("Error executing task list query: {}", e))?;

        for row_res in rows {
            let (id, task_text, task_priority, task_reward, uuid) = 
                row_res.map_err(|e| format_err!("Error deserializing task row from database: {}", e))?;
            let task = Task::from_parts(task_text, task_priority, task_reward, uuid)
                .map_err(|e| format_err!("Invalid task read from database row: {}", e))?;
            tasks.push((id, task));
        }
        Ok(tasks)
    }

    fn fetch_breaks(&self) -> Result<Vec<(RowId, Task)>, Error> {
        let tx = &self.transaction;

        let mut tasks = Vec::new();
        let mut stmt = tx.prepare_cached(
            "SELECT id, task, priority, category, uuid
            FROM tasks
            WHERE category = 1
            ORDER BY
             category ASC,
             priority ASC
            ")
            .map_err(|e| format_err!("Error preparing task list query: {}", e))?;
        let rows = stmt.query_map(NO_PARAMS, |row| {
                let row_id = RowId { id: row.get(0)?, _transaction: PhantomData };
                let task_text = row.get(1)?;
                let task_priority = row.get(2)?;
                let task_reward = row.get(3)?;
                let sql_uuid: SqlBlobUuid = row.get(4)?;

                Ok((row_id, task_text, task_priority, task_reward, sql_uuid.uuid))
             })
            .map_err(|e| format_err!("Error executing task list query: {}", e))?;

        for row_res in rows {
            let (id, task_text, task_priority, task_reward, uuid) = 
                row_res.map_err(|e| format_err!("Error deserializing task row from database: {}", e))?;
            let task = Task::from_parts(task_text, task_priority, task_reward, uuid)
                .map_err(|e| format_err!("Invalid task read from database row: {}", e))?;
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

    fn pop_current_task(&self) -> Result<Option<(RowId, Task)>, Error> {
        let tx = &self.transaction;
        let mut stmt = tx.prepare_cached(
            "SELECT id, task, priority, category, uuid
            FROM tasks
            WHERE id = (
                SELECT task_id FROM current
                WHERE id = 1
            )")
            .map_err(|e| format_err!("Error preparing pop current task query: {}", e))?;

        let rows: Vec<SQLResult<(RowId, String, u32, bool, Uuid)>> = stmt.query_map(NO_PARAMS, |row| {
                let row_id = RowId { id: row.get(0)?, _transaction: PhantomData };
                let task_text = row.get(1)?;
                let task_priority = row.get(2)?;
                let task_reward = row.get(3)?;
                let sql_uuid: SqlBlobUuid = row.get(4)?;

                Ok((row_id, task_text, task_priority, task_reward, sql_uuid.uuid))
             })
            .map_err(|e| format_err!("Error executing pop current task query: {}", e))?
            .collect();

        // no rows -> no current task, return None
        if rows.is_empty() {
            return Ok(None);
        }
        if rows.len() > 1 {
            return Err(format_err!("Multiple tasks selected in pop current task query. {} tasks, selected {:?}", rows.len(), rows))
        }

        let current_rowid_task: (RowId, Task) = rows.into_iter().next()
            .expect("No rows even though we checked there was one")
            .map(|t| Ok((t.0, Task::from_parts(t.1, t.2, t.3, t.4)?)))
            .map_err(|e| format_err!("Error deserializing task row from database: {}", e))?
            .map_err(|e: Error| format_err!("Invalid task was read from database row: {}", e))?;

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

        Ok(Some(current_rowid_task))
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

    fn try_remove_task_by_uuid(&self, uuid: &Uuid) -> Result<(), Error> {
        let tx = &self.transaction;
        let uuid_bytes: &[u8] = uuid.as_bytes();
        let rows_modified = tx.execute_named(
            "DELETE FROM tasks
            WHERE
                uuid = :task_uuid",
            &[(":task_uuid", &uuid_bytes)])
            .map_err(|e| format_err!("Error deleting task: {}", e))?;
        if rows_modified > 1 {
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
        Ok(())
    }
}

#[cfg(test)]
mod tests;
