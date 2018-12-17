use crate::db::DBTransaction;
use crate::db::tests::open_test_db;

use crate::task::Task;
use crate::task::test_utils::{example_task_1, example_task_break_1, example_task_list, arb_task_list_bounded};

#[test]
/// Add task, commit, remove task, check no task. Rollback, check task is there.
fn test_tx_remove_task() {
    let task = example_task_1();

    let mut db = open_test_db();

    let tx = db.transaction().unwrap();
    tx.add_task(&task).expect("Adding task failed");
    tx.commit().expect("Commiting failed");

    let tx = db.transaction().unwrap();
    let tasks = tx.get_tasks().expect("Getting tasks failed");

    let first_id = tasks[0].0;
    let res = tx.remove_task(&first_id);
    assert!(res.is_ok(), "Removing task failed: {}", res.unwrap_err());

    let tasks = tx.get_tasks().expect("Getting tasks failed");
    assert!(tasks.len() == 0, "Incorrect number of tasks in db, expected 0, got {}", tasks.len());

    tx.rollback().expect("Rolling back failed");

    let tx = db.transaction().unwrap();
    let tasks = tx.get_tasks().expect("Getting tasks failed");
    assert!(tasks.len() == 1, "Incorrect number of tasks in db, expected 1, got {}", tasks.len());
}

#[test]
/// Add all tasks in example list, remove and commit one at a time and make sure correct task is
/// removed. We do this by checking that all of the other tasks are still there, because we can
/// have duplicates in the db. We could compare RowIds but that's fragile. TODO: add UUIDs to
/// tasks and check those.
fn test_tx_remove_task_list() {
    let example_tasks = example_task_list();

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    // add tasks
    for t in &example_tasks {
        tx.add_task(t).expect("Adding task failed");
    }
    tx.commit().expect("Commiting failed");

    // remove each task and check that the task isn't in the db
    let mut remaining_tasks = example_tasks.clone();
    while remaining_tasks.len() > 0 {
        { // scope for remaining_db_tasks, which has a reference to the db
        let tx = db.transaction().unwrap();
        let db_tasks = tx.get_tasks().expect("Getting tasks failed");
        let db_breaks = tx.get_breaks().expect("Getting breaks failed");
        let remaining_db_tasks: Vec<_> = db_tasks.into_iter()
            .chain(db_breaks.into_iter())
            .collect();
        let (rowid, task) = &remaining_db_tasks[0];

        let res = tx.remove_task(&rowid);
        assert!(res.is_ok(), "Removing task failed: {}", res.unwrap_err());
        remove_task(&mut remaining_tasks, &task);
        tx.commit().unwrap();
        
        } // end remaining_db_tasks scope

        let tx = db.transaction().unwrap();
        let db_tasks = tx.get_tasks().expect("Getting tasks failed");
        let db_breaks = tx.get_breaks().expect("Getting breaks failed");
        let remaining_db_tasks: Vec<_> = db_tasks.into_iter().map(|t| t.1)
            .chain(db_breaks.into_iter().map(|t| t.1))
            .collect();

        assert!(remaining_db_tasks.len() == remaining_tasks.len(), 
            "Number of tasks in db isn't correct after removing one: expected {} got {}",
            remaining_tasks.len(), remaining_db_tasks.len());
        for in_task in &remaining_tasks {
            assert!(remaining_db_tasks.contains(&in_task),
                "Task {:?} not found in db tasks", in_task);
        }
    }
}

#[test]
/// Make sure an error is returned if we delete the task that's set as the current task.
fn test_tx_remove_task_current_empty_db() {
    let task = example_task_1();
    let brk = example_task_break_1();

    let mut db = open_test_db();

    let tx = db.transaction().unwrap();
    tx.add_task(&task).expect("Adding task failed");
    tx.add_task(&brk).expect("Adding task failed");
    
    let db_tasks = tx.get_tasks().expect("Getting tasks failed");
    let db_breaks = tx.get_breaks().expect("Getting breaks failed");
    let all_db_tasks: Vec<_> = db_tasks.into_iter().chain(db_breaks).collect();

    // set current task to first one
    let id = all_db_tasks[0].0;
    tx.set_current_task(&id).expect("Failed to set current task");
    // try removing it, get error
    let res = tx.remove_task(&id);
    assert!(res.is_err(), "Removed current task without error when we expected one.");
    let err = res.unwrap_err();
    assert!(err.to_string().contains("FOREIGN KEY constraint failed"), "Wrong error encountered: {}", err);

    // transaction is already rolled back at this point in sqlite internally? TODO figure out what's
    // happening on rusqlite side
    tx.rollback().unwrap();

    // assert db is empty now, as the tasks and removal were in the same transaction
    let tx = db.transaction().unwrap();
    let db_tasks = tx.get_tasks().expect("Getting tasks failed");
    let db_breaks = tx.get_breaks().expect("Getting breaks failed");
    let all_db_tasks: Vec<_> = db_tasks.into_iter().chain(db_breaks).collect();

    assert!(all_db_tasks.len() == 0, "Tasks were in db even after rolling back: {:?}", all_db_tasks);
}

/// Make sure an error is returned if we delete the task that's set as the current task. Same as
/// above, but commit before trying to remove current task.
#[test]
fn test_tx_remove_task_current_with_tasks() {
    let task = example_task_1();
    let brk = example_task_break_1();

    let mut db = open_test_db();

    let tx = db.transaction().unwrap();
    tx.add_task(&task).expect("Adding task failed");
    tx.add_task(&brk).expect("Adding task failed");
    tx.commit().unwrap();


    let tx = db.transaction().unwrap();
    let db_tasks = tx.get_tasks().expect("Getting tasks failed");
    let db_breaks = tx.get_breaks().expect("Getting breaks failed");
    let all_db_tasks: Vec<_> = db_tasks.into_iter().chain(db_breaks).collect();

    // set current task to second one
    let id = all_db_tasks[1].0;
    tx.set_current_task(&id).expect("Failed to set current task");
    // try removing it, get error
    let res = tx.remove_task(&id);
    assert!(res.is_err(), "Removed current task without error when we expected one.");
    let err = res.unwrap_err();
    assert!(err.to_string().contains("FOREIGN KEY constraint failed"), "Wrong error encountered: {}", err);

    // transaction is already rolled back at this point in sqlite internally? TODO figure out what's
    // happening on rusqlite side
    tx.rollback().unwrap();

    // assert db is empty now, as the tasks and removal were in the same transaction
    let tx = db.transaction().unwrap();
    let db_tasks = tx.get_tasks().expect("Getting tasks failed");
    let db_breaks = tx.get_breaks().expect("Getting breaks failed");
    let all_db_tasks: Vec<_> = db_tasks.into_iter().chain(db_breaks).collect();

    assert!(all_db_tasks.len() == 2, "Tasks were removed from db even though we rolled back: {:?}", all_db_tasks);
}

proptest! {
    /// Add all tasks in example list, remove and commit one at a time and make sure correct task is
    /// removed. We do this by checking that all of the other tasks are still there, because we can
    /// have duplicates in the db. We could compare RowIds but that's fragile. TODO: add UUIDs to
    /// tasks and check those.
    #[test]
    fn test_tx_remove_task_arb(example_tasks in arb_task_list_bounded()) {
        let mut db = open_test_db();
        let tx = db.transaction().unwrap();

        // add tasks
        for t in &example_tasks {
            tx.add_task(t).expect("Adding task failed");
        }
        tx.commit().expect("Commiting failed");

        // remove each task and check that the task isn't in the db
        let mut remaining_tasks = example_tasks.clone();
        while remaining_tasks.len() > 0 {
            { // scope for remaining_db_tasks, which has a reference to the db
            let tx = db.transaction().unwrap();
            let db_tasks = tx.get_tasks().expect("Getting tasks failed");
            let db_breaks = tx.get_breaks().expect("Getting breaks failed");
            let remaining_db_tasks: Vec<_> = db_tasks.into_iter()
                .chain(db_breaks.into_iter())
                .collect();
            let (rowid, task) = &remaining_db_tasks[0];

            let res = tx.remove_task(&rowid);
            prop_assert!(res.is_ok(), "Removing task failed: {}", res.unwrap_err());
            remove_task(&mut remaining_tasks, &task);
            tx.commit().unwrap();
            
            } // end remaining_db_tasks scope

            let tx = db.transaction().unwrap();
            let db_tasks = tx.get_tasks().expect("Getting tasks failed");
            let db_breaks = tx.get_breaks().expect("Getting breaks failed");
            let remaining_db_tasks: Vec<_> = db_tasks.into_iter().map(|t| t.1)
                .chain(db_breaks.into_iter().map(|t| t.1))
                .collect();

            prop_assert!(remaining_db_tasks.len() == remaining_tasks.len(), 
                "Number of tasks in db isn't correct after removing one: expected {} got {}",
                remaining_tasks.len(), remaining_db_tasks.len());
            for in_task in &remaining_tasks {
                prop_assert!(remaining_db_tasks.contains(&in_task),
                    "Task {:?} not found in db tasks", in_task);
            }
        }
    }
}


// utility function because remove_item/remove_by isn't stable for some reason
fn remove_task(tasks: &mut Vec<Task>, task: &Task) {
    let idx = tasks.iter()
        .enumerate()
        .filter(|(_, t)| t == &task)
        .map(|(i, _)| i)
        .next()
        .expect("no matching task found");
    tasks.remove(idx);
}
