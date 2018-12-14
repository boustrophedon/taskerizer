use failure::Error;
use crate::db::{SqliteBackend, DBMetadata};
use crate::db::DBTransaction;

use crate::task::Task;

use rusqlite::Result as SQLResult;

pub trait DBBackend {
    /// Get metadata about database
    fn metadata(&mut self) -> Result<DBMetadata, Error>;

    /// Add task to database
    fn add_task(&mut self, task: &Task) -> Result<(), Error>;

    /// Return a `Vec` of all tasks from the database
    fn get_all_tasks(&mut self) -> Result<Vec<Task>, Error>;

    /// Choose a new task at random to be the current task. Note that if the existing current task is not
    /// removed, it may be selected again. `p` is a parameter that must be between 0 and 1 which
    /// represents a probability, used to select the task with the `select_task` function. The
    /// `reward` parameter determines whether the task selected is selected from the break tasks or
    /// the regular tasks.
    ///
    /// TODO: specify the ordering more precisely by making the secondary sort key after priority
    /// something like date added. currently the task list is sorted by priority and then text.
    fn choose_current_task(&mut self, p: f32, reward: bool) -> Result<(), Error>;

    /// Returns the currently selected task if there is one, or None if there are no tasks in the
    /// database.  This function should never return None if there are tasks in the database.
    fn get_current_task(&mut self) -> Result<Option<Task>, Error>;

    /// Close the database. This is not really required due to the implementation of Drop for the
    /// Sqlite connection, but it might be necessary for other implementations e.g. a mock.
    fn close(self) -> Result<(), Error>;
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
    fn metadata(&mut self) -> Result<DBMetadata, Error> {
        // NOTE: even though we only read here, we are using a transaction so that when we switch
        // to implementing these operations in terms of transactions, everything should "just work"
        // simply by changing self to be a transaction wrapper. i.e. instead of "self.connection"
        // we'll just use "self.transaction" directly.
        let tx = self.connection.transaction()?;

        let (version, date_created) = tx.query_row(
            "SELECT version, date_created FROM metadata WHERE id = 1",
            &[],
            |row| {
                let version = row.get(0);
                let date_created = row.get(1);
                (version, date_created)
            }
        ).map_err(|e| format_err!("Error getting metadata from database: {}", e))?;

        tx.commit()?;
        Ok(
            DBMetadata {
                version: version,
                date_created: date_created,
            }
        )
    }

    fn add_task(&mut self, task: &Task) -> Result<(), Error> {
        let tx = self.transaction()?;
        tx.add_task(task)?; 
        tx.commit()?;
        Ok(())
    }

    fn get_all_tasks(&mut self) -> Result<Vec<Task>, Error> {
        let tx = self.transaction()?;
        let tasks = tx.get_tasks()
            .map_err(|e| format_err!("Failed to get tasks during transaction: {}", e))?;
        let breaks = tx.get_breaks()
            .map_err(|e| format_err!("Failed to get break tasks during transaction: {}", e))?;

        // chain tasks followed by breaks into single vector
        let all_tasks = tasks.into_iter().map(|t| t.1)
            .chain(breaks.into_iter().map(|t| t.1))
            .collect();

        tx.commit()?;
        return Ok(all_tasks);
    }

    fn choose_current_task(&mut self, p: f32, reward: bool) -> Result<(), Error> {
        if p < 0.0 {
            return Err(format_err!("p parameter was less than 0"));
        }
        if p > 1.0 {
            return Err(format_err!("p parameter was greater than 1"));
        }

        let tx = self.transaction()?;
        let tasks = {
            if reward {
                tx.get_breaks()
                    .map_err(|e| format_err!("Failed to get break tasks during transaction: {}", e))?
            }
            else {
                tx.get_tasks()
                    .map_err(|e| format_err!("Failed to get tasks during transaction: {}", e))?
            }
        };

        if tasks.len() == 0 {
            return Err(format_err!("No tasks with given category were found in the database to choose from."));
        }

        let selected_task_id = select_task(p, &tasks);

        tx.set_current_task(&selected_task_id)
            .map_err(|e| format_err!("Failed to set current task during transaction: {}", e))?;

        tx.commit()?;

        Ok(())
    }

    fn get_current_task(&mut self) -> Result<Option<Task>, Error> {
        let tx = self.connection.transaction()?;

        // we will be able to return the task directly without declaring outside the scope once we
        // move this function to TxOperations
        let current_task;

        { // begin transaction scope
        let mut stmt = tx.prepare_cached(
            "SELECT task, priority, category
            FROM tasks
            WHERE id = (
                SELECT task_id FROM current
                WHERE id = 1
            )
            ")
            .map_err(|e| format_err!("Error preparing current task query: {}", e))?;

        // TODO there should probably be a better way to do this.
        let mut rows: Vec<SQLResult<Result<Task, Error>>> = stmt.query_map(&[], |row| {
                Task::from_parts(row.get(0), row.get(1), row.get(2))
             })
            .map_err(|e| format_err!("Error executing current task query: {}", e))?
            .collect();

        if rows.len() == 0 {
            return Ok(None);
        }

        if rows.len() > 1 {
            return Err(format_err!("Multiple tasks selected in current task query. {} tasks, selected {:?}", rows.len(), rows))
        }

        // unwrap is fine, we check that there is one element directly above
        current_task = rows.pop().unwrap()
            .map_err(|e| format_err!("Error deserializing task row from database: {}", e))?
            .map_err(|e| format_err!("Invalid task read from database row: {}", e))?;
        } // end transaction scope
        tx.commit()?;
        Ok(Some(current_task))
    }

    fn close(self) -> Result<(), Error> {
        self.connection.close().map_err(|(_,e)| e.into())
    }
}

// Could make this more generic by taking a generic iterator, and two functions as parameters that
// select the weight and the item to be returned from the Item type in the iterator.
/// Given a float `p` between 0 and 1, and a slice of `Task`s, choose the task such that, when all
/// the priorities are added up and divided into intervals proportional to each `Task`s priority,
/// choose the Task whose interval `p` lies in. In pretty much all use cases we will have to
/// handled zero tasks separately anyway, so we panic if `tasks` is empty.
fn select_task<T: Copy>(p: f32, tasks: &[(T, Task)]) -> T {
    debug_assert!(0.0 <= p);
    debug_assert!(p <= 1.0);

    assert!(tasks.len() > 0, "Tasks is empty, nothing to select.");

    // we don't actually need the tasks to be ordered, though they will come in ordered
    //debug_assert!(tasks.windows(2).all(|t1, t2| t1.priority <= t2.priority))

    // TODO we convert the u32 into f32 to get around overflow issues but we could handle it
    // better
    let total_priority = tasks.iter().fold(0f32, |acc, (_, task)| acc + task.priority() as f32);

    let mut current_interval = 0.0;
    for (id, task) in tasks {
        current_interval += task.priority() as f32/total_priority;
        if p <= current_interval {
            return *id;
        }
    }
    // very unlikely, but due to rounding error we could choose p=0.99999... and the sum might
    // round to 0.99998 at the end, so we return the last task as a best effort. This line will
    // probably never be reached ever.
    let len = tasks.len();
    return tasks[len-1].0;
}


#[cfg(test)]
mod select_tests {
    use crate::task::Task;
    use crate::task::test_utils::{example_task_1, example_task_3, example_task_list, arb_task_list_bounded};
    use super::select_task;
    use proptest::test_runner::TestCaseError;

    #[test]
    fn test_select_task_single() {
        let tasks = vec![(1, example_task_1())];

        assert_eq!(select_task(0.0, &tasks), 1);
        assert_eq!(select_task(0.5, &tasks), 1);
        assert_eq!(select_task(1.0, &tasks), 1);
    }

    #[test]
    fn test_select_task_two() {
        let tasks = vec![(1, example_task_1()), (2, example_task_3())];

        assert_eq!(select_task(0.0, &tasks), 1);
        assert_eq!(select_task(0.2, &tasks), 1);
        assert_eq!(select_task(0.34, &tasks), 2);
        assert_eq!(select_task(0.5, &tasks), 2);
        assert_eq!(select_task(1.0, &tasks), 2);
    }

    fn prop_assert_select_in_order(tasks: &[(i32, Task)]) -> Result<(), TestCaseError> {
        // kind of ugly test because we're duplicating a computation inside the actual function
        let total_priority = tasks.iter().fold(0, |acc, (_, task)| acc + task.priority()) as f32;
        let min_sep = 1.0/total_priority/2.0;

        // walk through tasks by selecting them at an interval smaller than the smallest selection
        // interval size for a task.
        // make sure that they come out in the same order as the original list.

        let mut accum = 0.0;
        let mut orig_idx = 0;
        let mut selected = vec![select_task(accum, &tasks)];
        while accum <= 1.0 {
            let current = select_task(accum, &tasks);
            if current != tasks[orig_idx].0 {
                selected.push(current);
                orig_idx+=1;
            }

            accum+=min_sep;
        }

        let ids: Vec<i32> = tasks.iter().map(|(id, _)| *id).collect();
        prop_assert_eq!(selected, ids);
        Ok(())
    }

    #[test]
    fn test_select_task_list() {
        let tasks = example_task_list();
        let len = tasks.len() as i32;

        let tasks: Vec<(i32, Task)> = (1..len+1).zip(tasks).collect();
        assert!(prop_assert_select_in_order(&tasks).is_ok(), "Tasks are not selected in order of priority");
    }

    use proptest::test_runner::Config;
    proptest! {
        #![proptest_config(Config::with_cases(10))]
        #[test]
        fn test_select_task_list_arb(tasks in arb_task_list_bounded()) {
            let len = tasks.len() as i32;
            let tasks: Vec<(i32, Task)> = (1..len+1).zip(tasks).collect();
            prop_assert_select_in_order(&tasks)?;
        }
    }
}
