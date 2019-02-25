// want to keep RowId usage as pass-by-ref
#![allow(clippy::trivially_copy_pass_by_ref)]

use std::marker::PhantomData;

use failure::Error;
use rusqlite::NO_PARAMS;
use rusqlite::types::{FromSql, FromSqlResult, FromSqlError, ValueRef};
use uuid::Uuid;

use crate::db::SqliteTransaction;

use crate::task::Task;
use crate::sync::{USetOp, USetOpMsg, ClientUuid};

// TODO: rusqlite has a FromSql<i128> but not u128, whereas Uuid has From<u128> but not From<i128>.
// so add a FromSql<u128> to rusqlite.
struct SqlBlobUuid {
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
    /// Add task to database
    fn add_task(&self, task: &Task) -> Result<(), Error>;

    /// Return a `Vec` of all tasks (non-breaks) from the database. 
    fn fetch_tasks(&self) -> Result<Vec<(RowId, Task)>, Error>;

    /// Return a `Vec` of all breaks from the database.
    fn fetch_breaks(&self) -> Result<Vec<(RowId, Task)>, Error>;

    /// Set the current task to be the task with id `id`.
    fn set_current_task(&self, id: &RowId) -> Result<(), Error>;

    /// Returns the currently selected task if there is one, or None if there are no tasks in the
    /// database. This function should never return None if there are tasks in the database.
    fn fetch_current_task(&self) -> Result<Option<Task>, Error>;

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
    fn remove_task_by_uuid(&self, uuid: &Uuid) -> Result<(), Error>;

    /// Store an unsynced `USetOpMsg` in the database to transmit later.
    fn store_uset_op_msg(&self, uset_op_msg: &USetOpMsg) -> Result<(), Error>;

    /// Fetch all unsynced `USetOpMsg`s directed to a given client.
    fn fetch_uset_op_msgs(&self, client_id: &ClientUuid) -> Result<Vec<USetOpMsg>, Error>;

    /// Clear all unsynced `USetOpMsg`s directed to a given client.
    fn clear_uset_op_msgs(&self, client_id: &ClientUuid) -> Result<(), Error>;

    /// Add a server to the replica set. This does not do any network communication, it just stores
    /// the data.
    // TODO maybe use an actual url type for the url parameter?
    fn store_replica_server(&self, api_url: &str, replica_id: &ClientUuid) -> Result<(), Error>;

    /// Fetch all known replica servers from db
    // TODO maybe use an actual url type for the urls, use ReplicaServer type instead of tuple
    fn fetch_replica_servers(&self) -> Result<Vec<(String, Uuid)>, Error>;

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
        let uuid_bytes: &[u8] = task.uuid().as_bytes();

        tx.execute_named(
            "INSERT INTO tasks (task, priority, category, uuid) VALUES (:task, :priority, :category, :uuid)",
            &[(":task", &task.task()),
              (":priority", &task.priority()),
              (":category", &task.is_break()),
              (":uuid", &uuid_bytes)
            ],
        ).map_err(|e| format_err!("Error inserting task into database: {}", e))?;
        Ok(())
    }

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
                let rowid = RowId { id: row.get(0), _transaction: PhantomData };
                let sql_uuid: SqlBlobUuid = row.get(4);
                let task = Task::from_parts(row.get(1), row.get(2), row.get(3), sql_uuid.uuid);
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
                let rowid = RowId { id: row.get(0), _transaction: PhantomData };
                let sql_uuid: SqlBlobUuid = row.get(4);
                let task = Task::from_parts(row.get(1), row.get(2), row.get(3), sql_uuid.uuid);
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

    fn fetch_current_task(&self) -> Result<Option<Task>, Error> {
        let tx = &self.transaction;
        let mut stmt = tx.prepare_cached(
            "SELECT task, priority, category, uuid
            FROM tasks
            WHERE id = (
                SELECT task_id FROM current
                WHERE id = 1
            )
            ")
            .map_err(|e| format_err!("Error preparing current task query: {}", e))?;

        let rows: Vec<Result<Task, Error>> = stmt.query_map(NO_PARAMS, |row| {
                let sql_uuid: SqlBlobUuid = row.get(3);
                Ok(
                Task::from_parts(row.get(0), row.get(1), row.get(2), sql_uuid.uuid)
                    .map_err(|e| format_err!("Invalid task was read from database row: {}", e))?
                )
             })
            .map_err(|e| format_err!("Error executing current task query: {}", e))?
            .flat_map(|r| r)
            .collect();

        // No rows -> no current task
        if rows.is_empty() {
            return Ok(None);
        }

        if rows.len() > 1 {
            return Err(format_err!("Multiple tasks selected in current task query. {} tasks, selected {:?}", rows.len(), rows))
        }

        let current_task = rows.into_iter().next()
            .expect("No rows even though we checked there was one")
            .map_err(|e| format_err!("Error deserializing task row from database: {}", e))?;

        Ok(Some(current_task))
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

        let rows: Vec<Result<(RowId, Task), Error>> = stmt.query_map(NO_PARAMS, |row| {
                let sql_uuid: SqlBlobUuid = row.get(4);
                Ok((RowId { id: row.get(0), _transaction: PhantomData },
                Task::from_parts(row.get(1), row.get(2), row.get(3), sql_uuid.uuid)
                    .map_err(|e| format_err!("Invalid task was read from database row: {}", e))?
                ))
             })
            .map_err(|e| format_err!("Error executing pop current task query: {}", e))?
            .flat_map(|r| r)
            .collect();

        // no rows -> no current task, return None
        if rows.is_empty() {
            return Ok(None);
        }
        if rows.len() > 1 {
            return Err(format_err!("Multiple tasks selected in pop current task query. {} tasks, selected {:?}", rows.len(), rows))
        }

        let current_rowid_task = rows.into_iter().next()
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

    fn remove_task_by_uuid(&self, uuid: &Uuid) -> Result<(), Error> {
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

    fn store_uset_op_msg(&self, uset_op_msg: &USetOpMsg) -> Result<(), Error> {
        let tx = &self.transaction;
        let client_uuid_bytes: &[u8] = uset_op_msg.deliver_to.as_bytes();

        match &uset_op_msg.op {
            USetOp::Add(task) => {
                let uuid_bytes: &[u8] = task.uuid().as_bytes();
                tx.execute_named(
                    "INSERT INTO unsynced_ops
                    (is_add_operation, task, priority, category, task_uuid, client_uuid)
                    VALUES (:is_add_operation, :task, :priority, :category, :task_uuid, :client_uuid)",
                    &[(":is_add_operation", &true),
                      (":task", &task.task()),
                      (":priority", &task.priority()),
                      (":category", &task.is_break()),
                      (":task_uuid", &uuid_bytes),
                      (":client_uuid", &client_uuid_bytes)
                    ],
                ).map_err(|e| format_err!("Error inserting task into database: {}", e))?;
            },
            USetOp::Remove(task_uuid) => {
                let uuid_bytes: &[u8] = task_uuid.as_bytes();
                tx.execute_named(
                    "INSERT INTO unsynced_ops
                    (is_add_operation, task_uuid, client_uuid)
                    VALUES (:is_add_operation, :task_uuid, :client_uuid)",
                    &[(":is_add_operation", &false),
                      (":task_uuid", &uuid_bytes),
                      (":client_uuid", &client_uuid_bytes)
                    ],
                ).map_err(|e| format_err!("Error inserting task into database: {}", e))?;
            }
        }

        Ok(())
    }

    fn fetch_uset_op_msgs(&self, client_id: &ClientUuid) -> Result<Vec<USetOpMsg>, Error> {
        let tx = &self.transaction;
        let client_uuid_bytes: &[u8] = client_id.as_bytes();

        let mut stmt = tx.prepare_cached(
            "SELECT is_add_operation, task, priority, category, task_uuid, client_uuid
            FROM unsynced_ops
            WHERE client_uuid = :client_uuid
            ORDER BY id
            ")
            .map_err(|e| format_err!("Error preparing current task query: {}", e))?;

        let rows = stmt.query_map(&[&client_uuid_bytes,], |row| {
                let is_add = row.get(0);
                if is_add {
                    let sql_task_uuid: SqlBlobUuid = row.get(4);
                    let sql_client_uuid: SqlBlobUuid = row.get(5);
                    let task = Task::from_parts(row.get(1), row.get(2), row.get(3), sql_task_uuid.uuid)
                        .map_err(|e| format_err!("Invalid task was read from database row: {}", e))?;
                    let op = USetOp::Add(task);
                    let deliver_to = sql_client_uuid.uuid;

                    // type annotation to help compiler infer err type of result here,
                    // instead of writing out the full type of `rows`
                    let res: Result<USetOpMsg, Error> = Ok(USetOpMsg {op, deliver_to});
                    res
                }
                else {
                    let sql_task_uuid: SqlBlobUuid = row.get(4);
                    let sql_client_uuid: SqlBlobUuid = row.get(5);
                    let op = USetOp::Remove(sql_task_uuid.uuid);
                    let deliver_to = sql_client_uuid.uuid;

                    Ok(USetOpMsg {op, deliver_to})
                }
             })
            .map_err(|e| format_err!("Error executing current task query: {}", e))?;

        let mut msgs = Vec::new();
        // There are three kinds of errors that can occur:
        // The executing the query can error (eg syntax error)
        // Deserializing a row can error (internal error? like there's a null byte in a text column or something?)
        // Task::from_parts can error (eg priority=0) (this is why the actual value returned by the query map is a Result)
        for row_res in rows {
            let msg_result = row_res.map_err(|e| format_err!("Error deserializing msg row from unsynced_ops table: {}", e))?;
            msgs.push(msg_result?);
        }

        Ok(msgs)
    }

    fn clear_uset_op_msgs(&self, client_id: &ClientUuid) -> Result<(), Error> {
        let tx = &self.transaction;
        let client_uuid_bytes: &[u8] = client_id.as_bytes();
        tx.execute_named("DELETE FROM unsynced_ops
                         WHERE
                           client_uuid = :client_uuid",
                           &[(":client_uuid", &client_uuid_bytes)])
            .map_err(|e| format_err!("Error clearing unsyced ops: {}", e))?;
        Ok(())
    }

    fn store_replica_server(&self, api_url: &str, replica_id: &ClientUuid) -> Result<(), Error> {
        let tx = &self.transaction;

        let uuid_bytes: &[u8] = replica_id.as_bytes();
        tx.execute_named(
            "INSERT INTO replicas (replica_uuid) VALUES (:replica_uuid)",
            &[(":replica_uuid", &uuid_bytes),
            ],
        ).map_err(|e| format_err!("Error inserting server id into databasease: {}", e))?;
        tx.execute_named(
            "INSERT INTO servers (api_url, replica_id) VALUES (:api_url, last_insert_rowid())",
            &[(":api_url", &api_url),
            ],
        ).map_err(|e| format_err!("Error inserting server url into databasease: {}", e))?;
        Ok(())
    }

    fn fetch_replica_servers(&self) -> Result<Vec<(String, Uuid)>, Error> {
        let tx = &self.transaction;

        let mut stmt = tx.prepare_cached(
            "SELECT servers.api_url, replicas.replica_uuid
            FROM servers
            INNER JOIN replicas
            ON servers.replica_id = replicas.id
            ")
            .map_err(|e| format_err!("Error preparing fetch replica servers query: {}", e))?;

        let rows = stmt.query_map(NO_PARAMS, |row| {
            let sql_uuid: SqlBlobUuid = row.get(1);
            (row.get(0), sql_uuid.uuid)
        })
        .map_err(|e| format_err!("Error fetching replica servers from database: {}", e))?;

        let mut servers: Vec<(String, Uuid)> = Vec::new();
        for server_res in rows {
            let server_data = server_res.map_err(|e| format_err!("Error deserializing row from database: {}", e))?;
            servers.push(server_data);
        }

        Ok(servers)
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
