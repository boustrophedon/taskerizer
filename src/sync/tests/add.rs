use std::collections::HashSet;

use crate::db::DBBackend;
use crate::task::Task;
use crate::sync::USetOp;

use crate::db::tests::open_test_db;
use crate::task::test_utils::{example_task_1, example_task_break_1, example_task_2};
use crate::sync::test_utils::uset_add_list_arb;

#[test]
/// Add via uset, check task is in list
fn test_uset_add_task_1() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let task = example_task_1();

    // add example task
    let op = USetOp::Add(task.clone());
    let res = op.apply_to_db(&tx);
    assert!(res.is_ok(), "Could not apply add operation to db: {}", res.unwrap_err());

    // read tasks from db and check it's there
    let db_tasks = tx.fetch_all_tasks().expect("Couldn't fetch tasks");
    assert_eq!(db_tasks[0], task);
}

#[test]
/// Add via uset, check task is in list
fn test_uset_add_break_1() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let task = example_task_break_1();

    // add example task
    let op = USetOp::Add(task.clone());
    let res = op.apply_to_db(&tx);
    assert!(res.is_ok(), "Could not apply add operation to db: {}", res.unwrap_err());

    // read tasks from db and check it's there
    let db_tasks = tx.fetch_all_tasks().expect("Could not get tasks");
    assert_eq!(db_tasks[0], task);
}

#[test]
/// Add test "locally", then add same task with same uuid via USetOp. Check for error.
fn test_uset_add_error_dup_uuid() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();
    let task = example_task_2();

    // add example task;
    tx.add_task(&task).expect("Failed adding test");

    let op = USetOp::Add(task.clone());
    let res = op.apply_to_db(&tx);
    assert!(res.is_err(), "No error adding task with duplicate uuid to db");

    let err = res.unwrap_err();
    assert!(err.to_string().contains("UNIQUE constraint failed: tasks.uuid"),
        "Incorrect error message when inserting duplicate task: got {}", err);
}

proptest! {
    #[test]
    /// Add tasks via USet, check they are all there.
    fn test_uset_add_arb(ops in uset_add_list_arb()) {
        let mut db = open_test_db();
        let tx = db.transaction().unwrap();

        // apply all add ops
        for op in &ops {
            let res = op.apply_to_db(&tx);
            prop_assert!(res.is_ok(), "Could not apply add operation to db: {}", res.unwrap_err());
        }

        let db_tasks = tx.fetch_all_tasks().expect("Couldn't fetch tasks");
        // check number of tasks in db is equal to number of add ops performed
        prop_assert_eq!(db_tasks.len(), ops.len());

        // put the ops into a hashset for faster checking - there shouldn't be any dups except for
        // small probability of uuid collision
        let tasks_set: HashSet<Task> = ops.into_iter().map(|op| op.unwrap_add()).collect();
        for db_task in &db_tasks {
            // check db task in tasks from add ops
            prop_assert!(tasks_set.contains(db_task));
        }
    }
}
