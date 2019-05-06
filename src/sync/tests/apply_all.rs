use crate::db::DBBackend;

use crate::sync::{apply_all_uset_ops, USetOp};

use crate::db::tests::open_test_db;
use crate::task::test_utils::{example_task_1, example_task_2, example_task_3};
use crate::sync::test_utils::{example_remove_uset_op_2};

// TODO: add proptests

#[test]
fn test_uset_apply_all_empty() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let tasks: Vec<USetOp> = Vec::new();

    let res = apply_all_uset_ops(&tx, &tasks);
    assert!(res.is_ok(), "Error applying all uset ops: {}", res.unwrap_err());

    let messages = res.unwrap();
    assert!(messages.is_empty(), "Messages resulted from applying no uset ops: {:?}", messages);
}

#[test]
/// Add two tasks via uset, make sure there are two tasks in db
fn test_uset_apply_all_2() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let tasks = vec![USetOp::Add(example_task_1()), USetOp::Add(example_task_2())];

    let res = apply_all_uset_ops(&tx, &tasks);
    assert!(res.is_ok(), "Error applying all uset ops: {}", res.unwrap_err());

    let messages = res.unwrap();
    assert!(messages.is_empty(), "Messages resulted from applying only add uset ops: {:?}", messages);
    tx.finish().unwrap();

    let tx = db.transaction().unwrap();
    let tasks = tx.fetch_all_tasks().expect("Failed to fetch all tasks");
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0], example_task_1());
    assert_eq!(tasks[1], example_task_2());
}

#[test]
/// Add duplicate tasks with the same task UUID and make sure there is an error, and that there are
/// no tasks in the database.
fn test_uset_apply_all_duplicate_error() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    // task_1 twice
    let tasks = vec![USetOp::Add(example_task_1()), USetOp::Add(example_task_1())];
   
    let res = apply_all_uset_ops(&tx, &tasks);
    assert!(res.is_err(), "No error when adding duplicate tasks with the same id");
    drop(tx);

    let tx = db.transaction().unwrap();
    let tasks = tx.fetch_all_tasks().expect("Failed to fetch all tasks");
    assert!(tasks.is_empty(), "Tasks in database after aborted transaction: {:?}", tasks);
}

#[test]
/// Populate db with 1 task, then try to add duplicate tasks with the same task UUID and make sure
/// there is an error, and that only the pre-existing tasks in the database remain.
fn test_uset_apply_all_duplicate_error_keeps_existing_task() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    tx.add_task(&example_task_2()).unwrap();
    tx.finish().unwrap();

    let tx = db.transaction().unwrap();
    // task_1 twice
    let tasks = vec![USetOp::Add(example_task_3()), USetOp::Add(example_task_1()), USetOp::Add(example_task_1())];

    let res = apply_all_uset_ops(&tx, &tasks);
    assert!(res.is_err(), "No error when adding duplicate tasks with the same id");
    drop(tx);

    let tx = db.transaction().unwrap();
    let tasks = tx.fetch_all_tasks().expect("Failed to fetch all tasks");
    assert_eq!(tasks, vec![example_task_2(),]);
}

#[test]
/// Removing a non-existant task should not raise an error.
fn test_uset_apply_all_remove_non_existant() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    // add task_1, remove task2
    let tasks = vec![USetOp::Add(example_task_1()), example_remove_uset_op_2()];

    let res = apply_all_uset_ops(&tx, &tasks);
    assert!(res.is_ok(), "Unexpected error when removing non-existant task via uset op: {}");
    tx.finish().unwrap();

    // check task_1 is in task list
    let tx = db.transaction().unwrap();
    let tasks = tx.fetch_all_tasks().expect("Failed to fetch all tasks");
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0], example_task_1());
}

#[test]
/// Removing the currently-set task during the sequence of operations should return a message.
fn test_uset_apply_all_remove_current_task() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let mut top = crate::selection::Top::new();

    // add task_2 and select it as the current task
    tx.add_task(&example_task_2()).expect("Failed to add task");
    tx.select_current_task(&mut top).expect("Failed to select current task");

    // remove task2, add task1
    let tasks = vec![example_remove_uset_op_2(), USetOp::Add(example_task_1())];

    let res = apply_all_uset_ops(&tx, &tasks);
    assert!(res.is_ok(), "Unexpected error when removing non-existant task via uset op: {}");

    let messages = res.unwrap();
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("Current task removed during sync:"));
    assert!(messages[0].contains(example_task_2().task()));
}
