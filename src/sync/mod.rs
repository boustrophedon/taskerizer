use crate::task::Task;
use crate::db::DBBackend;

use failure::Error;
use uuid::Uuid;

pub type ClientUuid = Uuid;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum USetOp {
    Add (Task),
    Remove (Uuid),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A message containing a U-Set operation and the recipient.
struct USetOpMsg {
    op: USetOp,
    deliver_to: ClientUuid,
}

impl USetOp {
    /// Apply the U-Set operation to the database. Returns a Result containing text to print to the
    /// user, including but not limited to a notification that the current task was removed while
    /// applying the operation.
    pub fn apply_to_db(&self, tx: &impl DBBackend) -> Result<Vec<String>, Error> {
        match self {
            USetOp::Add(task) => USetOp::apply_add_to_db(tx, task).map(|_| Vec::new()),
            USetOp::Remove(uuid) => {
                // FIXME: there's probably a simpler way of doing this than the nested maps,
                // probably using flat_map earlier.
                USetOp::apply_remove_to_db(tx, uuid)
                    .map(|opt_task| {
                        opt_task.iter().flat_map(|task| {
                            vec![format!("Current task removed during sync: {}", task.task())]
                        }).collect()
                    })
            }
        }
    }

    /// Apply a U-Set add operation to the database. This will error if a task with the same UUID
    /// exists in the database.
    fn apply_add_to_db(tx: &impl DBBackend, task: &Task) -> Result<(), Error> {
        tx.add_task(task)
    }

    /// Apply a U-Set remove operation to the database. If the task to be removed was the current task, it
    /// is returned so the user can be notified. If there is no task in the database with the given
    /// UUID, nothing happens.
    fn apply_remove_to_db(tx: &impl DBBackend, uuid: &Uuid) -> Result<Option<Task>, Error> {
        tx.remove_task_by_uuid(uuid)
    }
}
