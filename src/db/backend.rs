use failure::Error;
use rusqlite::NO_PARAMS;
use uuid::Uuid;

use crate::db::DBMetadata;
use crate::db::{SqliteTransaction, DBTransaction};
use crate::db::transaction::SqlBlobUuid;

use crate::selection::SelectionStrategy;

use crate::sync::{USetOp, USetOpMsg, ReplicaUuid};
use crate::task::{Category, Task};


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

    /// Replace the current task with a different one, leaving the previous current task in the database.
    fn skip_current_task(&self, selector: &mut dyn SelectionStrategy) -> Result<(), Error>;

    /// Remove the previous current task from the database and mark it as completed. This will
    /// leave the database without a current task.
    fn complete_current_task(&self) -> Result<Option<Task>, Error>;

    /// Remove a task from the database. If the task was set as the current task, it is unset as
    /// the current task and returned. Otherwise, the result is `Ok(None)`.
    ///
    /// This is used for network sync and isn't exposed in a CLI command currently.
    fn remove_task_by_uuid(&self, uuid: &Uuid) -> Result<Option<Task>, Error>;

    /// Store an unsynced `USetOpMsg` in the database to transmit later.
    fn store_uset_op_msg(&self, uset_op_msg: &USetOpMsg) -> Result<(), Error>;

    /// Fetch all unsynced `USetOpMsg`s directed to a given replica.
    fn fetch_uset_op_msgs(&self, replica_id: &ReplicaUuid) -> Result<Vec<USetOpMsg>, Error>;

    /// Clear all unsynced `USetOpMsg`s directed to a given replica.
    fn clear_uset_op_msgs(&self, replica_id: &ReplicaUuid) -> Result<(), Error>;

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
                version,
                date_created,
            }
        )
    }

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

        Ok(all_tasks)
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

    fn skip_current_task(&self, selector: &mut dyn SelectionStrategy) -> Result<(), Error> {
        let tx = self;

        let current_opt = tx.pop_current_task()
            .map_err(|e| format_err!("Failed to pop current task during transaction: {}", e))?;

        // if there isn't a current task selected, there are no tasks in the db
        let (old_current_task_id, old_current_task) = match current_opt {
            Some((id, task)) => (id, task),
            None => return Ok(()),
        };

        // The reason we don't just call `complete_current_task` here is because at some point I
        // want to make that add tasks to a `completed` db table, but obviously I don't here.
        tx.remove_task(&old_current_task_id)
            .map_err(|e| format_err!("Failed to remove task during transaction: {}", e))?;

        tx.select_current_task(selector)
            .map_err(|e| format_err!("Failed to select new current task during transaction: {}", e))?;

        SqliteTransaction::add_task(self, &old_current_task)
            .map_err(|e| format_err!("Failed to add original task back to the db during transaction: {}", e))?;

        Ok(())
    }

    /// Replace the current task with a new one, removing the previous current task from the
    /// database and returning it.
    fn complete_current_task(&self) -> Result<Option<Task>, Error> {
        let tx = self;

        let current_opt = tx.pop_current_task()
            .map_err(|e| format_err!("Failed to pop current task during transaction: {}", e))?;
        let (current_task_id, current_task) = match current_opt {
            Some((id, task)) => (id, task),
            None => return Ok(None),
        };

        tx.remove_task(&current_task_id)
            .map_err(|e| format_err!("Failed to remove task during transaction: {}", e))?;

        Ok(Some(current_task))
    }

    fn remove_task_by_uuid(&self, uuid: &Uuid) -> Result<Option<Task>, Error> {
        let tx = self;

        let current_opt = DBBackend::fetch_current_task(tx)
            .map_err(|e| format_err!("Failed to get current task during USet remove operation: {}", e))?;

        // If the task we're removing is the current task, use the complete operation
        if let Some(current_task) = current_opt {
            if current_task.uuid() == uuid {
                return tx.complete_current_task()
                    .map_err(|e| format_err!("Failed to complete current task when removing task: {}", e));
            }
        }

        // If there is no current task, remove task normally
        DBTransaction::try_remove_task_by_uuid(tx, uuid).map(|_| None)
    }

    fn store_uset_op_msg(&self, uset_op_msg: &USetOpMsg) -> Result<(), Error> {
        let tx = &self.transaction;
        let replica_uuid_bytes: &[u8] = uset_op_msg.deliver_to.as_bytes();

        match &uset_op_msg.op {
            USetOp::Add(task) => {
                let uuid_bytes: &[u8] = task.uuid().as_bytes();
                tx.execute_named(
                    "INSERT INTO unsynced_ops
                    (is_add_operation, task, priority, category, task_uuid, replica_uuid)
                    VALUES (:is_add_operation, :task, :priority, :category, :task_uuid, :replica_uuid)",
                    &[(":is_add_operation", &true),
                      (":task", &task.task()),
                      (":priority", &task.priority()),
                      (":category", &task.is_break()),
                      (":task_uuid", &uuid_bytes),
                      (":replica_uuid", &replica_uuid_bytes)
                    ],
                ).map_err(|e| format_err!("Error inserting task into database: {}", e))?;
            },
            USetOp::Remove(task_uuid) => {
                let uuid_bytes: &[u8] = task_uuid.as_bytes();
                tx.execute_named(
                    "INSERT INTO unsynced_ops
                    (is_add_operation, task_uuid, replica_uuid)
                    VALUES (:is_add_operation, :task_uuid, :replica_uuid)",
                    &[(":is_add_operation", &false),
                      (":task_uuid", &uuid_bytes),
                      (":replica_uuid", &replica_uuid_bytes)
                    ],
                ).map_err(|e| format_err!("Error inserting task into database: {}", e))?;
            }
        }

        Ok(())
    }

    fn fetch_uset_op_msgs(&self, replica_id: &ReplicaUuid) -> Result<Vec<USetOpMsg>, Error> {
        let tx = &self.transaction;
        let replica_uuid_bytes: &[u8] = replica_id.as_bytes();

        let mut stmt = tx.prepare_cached(
            "SELECT is_add_operation, task, priority, category, task_uuid, replica_uuid
            FROM unsynced_ops
            WHERE replica_uuid = :replica_uuid
            ORDER BY id
            ")
            .map_err(|e| format_err!("Error preparing current task query: {}", e))?;

        let rows = stmt.query_map(&[&replica_uuid_bytes,], |row| {
                let is_add = row.get(0);
                if is_add {
                    let sql_task_uuid: SqlBlobUuid = row.get(4);
                    let sql_replica_uuid: SqlBlobUuid = row.get(5);
                    let task = Task::from_parts(row.get(1), row.get(2), row.get(3), sql_task_uuid.uuid)
                        .map_err(|e| format_err!("Invalid task was read from database row: {}", e))?;
                    let op = USetOp::Add(task);
                    let deliver_to = sql_replica_uuid.uuid;

                    // type annotation to help compiler infer err type of result here,
                    // instead of writing out the full type of `rows`
                    let res: Result<USetOpMsg, Error> = Ok(USetOpMsg {op, deliver_to});
                    res
                }
                else {
                    let sql_task_uuid: SqlBlobUuid = row.get(4);
                    let sql_replica_uuid: SqlBlobUuid = row.get(5);
                    let op = USetOp::Remove(sql_task_uuid.uuid);
                    let deliver_to = sql_replica_uuid.uuid;

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

    fn clear_uset_op_msgs(&self, replica_id: &ReplicaUuid) -> Result<(), Error> {
        let tx = &self.transaction;
        let replica_uuid_bytes: &[u8] = replica_id.as_bytes();
        tx.execute_named("DELETE FROM unsynced_ops
                         WHERE
                           replica_uuid = :replica_uuid",
                           &[(":replica_uuid", &replica_uuid_bytes)])
            .map_err(|e| format_err!("Error clearing unsyced ops: {}", e))?;
        Ok(())
    }

    fn finish(self) -> Result<(), Error> {
        self.commit()
    }

}
