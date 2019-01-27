use std::collections::HashSet;

use crate::db::DBTransaction;
use crate::db::tests::open_test_db;

use crate::task::test_utils::{example_task_1, example_task_2, example_task_1_dup, example_task_break_1, example_task_list, arb_task_list_bounded};

// these tests are copied directly from transaction/tests/remove_task and just modified to use the
// uuid.

#[test]
/// Attempt to remove task that was not added, check it succeeds and no other tasks were removed.
fn test_db_remove_nonexistant_uuid() {
    let task1 = example_task_1();
    let task2_not_added = example_task_2();

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();
    tx.add_task(&task1).expect("Adding task failed.");

    let res = tx.remove_task_by_uuid(&task2_not_added.uuid());
    assert!(res.is_ok(), "Error removing task not inserted into db: {}", res.unwrap_err());

    let tasks = tx.fetch_tasks().expect("Getting tasks failed");
    assert_eq!(tasks[0].1, task1);
}

#[test]
/// Add task, commit, remove task, check no task.
fn test_db_remove_task_by_uuid() {
    let task = example_task_1();

    let mut db = open_test_db();

    let tx = db.transaction().unwrap();
    tx.add_task(&task).expect("Adding task failed");

    let res = tx.remove_task_by_uuid(&task.uuid());
    assert!(res.is_ok(), "Removing task failed: {}", res.unwrap_err());

    let tasks = tx.fetch_tasks().expect("Getting tasks failed");
    assert!(tasks.len() == 0, "Incorrect number of tasks in db, expected 0, got {}", tasks.len());
}

#[test]
fn test_db_remove_task_by_uuid_duplicates() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let task1 = example_task_1();
    let task2 = example_task_1_dup();

    // we insert the same task (with different uuids) twice
    tx.add_task(&task1).expect("Adding task failed");
    tx.add_task(&task2).expect("Adding task failed");

    // remove it, and then make sure that there's still one left
    let res = tx.remove_task_by_uuid(&task1.uuid());
    assert!(res.is_ok(), "Removing task failed: {}", res.unwrap_err());

    let tasks = tx.fetch_tasks().expect("Getting tasks failed");
    assert!(tasks.len() == 1, "Incorrect number of tasks in db, expected 1, got {}", tasks.len());
}

#[test]
/// Make sure an error is still returned if we delete the task that's set as the current task.
fn test_db_remove_task_by_uuid_current() {
    let task = example_task_1();
    let brk = example_task_break_1();

    let mut db = open_test_db();

    let tx = db.transaction().unwrap();
    tx.add_task(&task).expect("Adding task failed");
    tx.add_task(&brk).expect("Adding task failed");
    
    let db_tasks = tx.fetch_tasks().expect("Getting tasks failed");
    let db_breaks = tx.fetch_breaks().expect("Getting breaks failed");
    let all_db_tasks: Vec<_> = db_tasks.into_iter().chain(db_breaks).collect();

    // set current task to first one (which is `task`)
    let id = all_db_tasks[0].0;
    tx.set_current_task(&id).expect("Failed to set current task");
    // try removing it, get error
    let res = tx.remove_task_by_uuid(&task.uuid());
    assert!(res.is_err(), "Removed current task without error when we expected one.");
    let err = res.unwrap_err();
    assert!(err.to_string().contains("FOREIGN KEY constraint failed"), "Wrong error encountered: {}", err);
}

#[test]
/// Add all tasks in example list, remove and commit one at a time and make sure correct task is
/// removed. We do this by checking that all of the other tasks are still there, because we can
/// have duplicates in the db.
fn test_db_remove_task_by_uuid_list() {
    let example_tasks = example_task_list();

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    // add tasks
    for t in &example_tasks {
        tx.add_task(t).expect("Adding task failed");
    }

    // remove each task and check that the task isn't in the db
    let mut remaining_tasks: HashSet<_> = example_tasks.iter().collect();
    while remaining_tasks.len() > 0 {
        // pop a task from the remaining tasks, remove it from db
        let to_be_removed = remaining_tasks.iter().next().expect("pop failed when remaining_tasks had len>0").clone();
        remaining_tasks.remove(to_be_removed);

        let res = tx.remove_task_by_uuid(&to_be_removed.uuid());
        assert!(res.is_ok(), "Removing task failed: {}", res.unwrap_err());

        // get the remaining tasks from the db
        let db_tasks = tx.fetch_tasks().expect("Getting tasks failed");
        let db_breaks = tx.fetch_breaks().expect("Getting breaks failed");
        let remaining_db_tasks: HashSet<_> = db_tasks.into_iter()
            .chain(db_breaks.into_iter())
            .map(|t| t.1)
            .collect();

        // check that the number of tasks from the db is the same as the ones we have in
        // `remaining_tasks`
        assert!(remaining_db_tasks.len() == remaining_tasks.len(), 
            "Number of tasks in db isn't correct after removing one: expected {} got {}",
            remaining_tasks.len(), remaining_db_tasks.len());
        // check that all the tasks in `remaining_tasks` are in the db
        for in_task in &remaining_tasks {
            assert!(remaining_db_tasks.contains(&in_task),
                "Task {:?} not found in db tasks", in_task);
        }
    }
}

use proptest::test_runner::Config;
proptest! {
    #![proptest_config(Config::with_cases(75))]
    #[test]
    /// Add all tasks in example list, remove one at a time and make sure correct task is removed.
    fn test_db_remove_task_by_uuid_arb(example_tasks in arb_task_list_bounded()) {
        let mut db = open_test_db();
        let tx = db.transaction().unwrap();

        // add tasks
        for t in &example_tasks {
            tx.add_task(t).expect("Adding task failed");
        }

        // remove each task and check that the task isn't in the db
        let mut remaining_tasks: HashSet<_> = example_tasks.iter().collect();
        while remaining_tasks.len() > 0 {
            // pop a task from the remaining tasks, remove it from db
            let to_be_removed = remaining_tasks.iter().next().expect("pop failed when remaining_tasks had len>0").clone();
            remaining_tasks.remove(to_be_removed);

            let res = tx.remove_task_by_uuid(&to_be_removed.uuid());
            prop_assert!(res.is_ok(), "Removing task failed: {}", res.unwrap_err());

            // get the remaining tasks from the db
            let db_tasks = tx.fetch_tasks().expect("Getting tasks failed");
            let db_breaks = tx.fetch_breaks().expect("Getting breaks failed");
            let remaining_db_tasks: HashSet<_> = db_tasks.into_iter()
                .chain(db_breaks.into_iter())
                .map(|t| t.1)
                .collect();

            // check that the number of tasks from the db is the same as the ones we have in
            // `remaining_tasks`
            prop_assert!(remaining_db_tasks.len() == remaining_tasks.len(), 
                "Number of tasks in db isn't correct after removing one: expected {} got {}",
                remaining_tasks.len(), remaining_db_tasks.len());
            // check that all the tasks in `remaining_tasks` are in the db
            for in_task in &remaining_tasks {
                prop_assert!(remaining_db_tasks.contains(&in_task),
                    "Task {:?} not found in db tasks", in_task);
            }
        }
    }
}
