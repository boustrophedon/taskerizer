use crate::db::DBBackend;
use crate::db::tests::open_test_db;

use crate::sync::USetOp;
use crate::sync::server::{process_clear, process_sync};

use crate::sync::test_utils::{example_replica_1, example_replica_2, example_replica_3};
use crate::sync::test_utils::{example_add_uset_op_1, example_add_uset_op_2};

use crate::task::test_utils::{example_task_1};

use pretty_assertions::assert_eq;

// FIXME: there aren't any tests using a remove op (should probably do this) and there aren't any
// property tests (probably don't need to do this)

#[test]
/// Clear on empty database, check result is Ok
fn test_server_process_clear_empty() {
    let mut db = open_test_db();
    let mut tx = db.transaction().expect("Failed to open transaction");

    let replica = example_replica_1();

    let res = process_clear(&mut tx, replica);
    assert!(res.is_ok(), "Error processing clear on empty database: {}", res.unwrap_err());
}

#[test]
/// Clear on unknown replica id, check result is Ok.
fn test_server_process_clear_unknown_id() {
    let mut db = open_test_db();
    let mut tx = db.transaction().unwrap();

    let replica = example_replica_1();

    let res = process_clear(&mut tx, replica);
    assert!(res.is_ok(), "Error processing clear on unknown replica: {}", res.unwrap_err());
}

#[test]
/// Clear on new replica, check result is Ok
fn test_server_process_clear_no_tasks() {
    let mut db = open_test_db();
    let mut tx = db.transaction().unwrap();

    let replica = example_replica_1();
    process_sync(&mut tx, replica, &[]).unwrap();

    let res = process_clear(&mut tx, replica);
    assert!(res.is_ok(), "Error processing clear on empty database: {}", res.unwrap_err());
}

#[test]
/// Check that clear preserves tasks in database.
/// sync replica with new task, clear, sync again and check nothing is returned but the task is
/// still in the database.
fn test_server_process_clear_preserve_database() {
    let mut db = open_test_db();
    let mut tx = db.transaction().unwrap();

    let replica = example_replica_1();
    let ops = vec![example_add_uset_op_1(),];

    let resp_ops = process_sync(&mut tx, replica, &ops).expect("failed to process sync");
    assert!(resp_ops.is_empty());

    let res = process_clear(&mut tx, replica);
    assert!(res.is_ok(), "Error processing clear after sync: {}", res.unwrap_err());

    let tasks = tx.fetch_all_tasks().unwrap();
    assert!(tasks.len() == 1, "Unexpected tasks in db: expected 1, got {:?}", tasks);

    assert_eq!(example_task_1(), tasks[0]);
}

#[test]
/// Check that clear actually clears unsynced tasks.
/// Register replica 1, sync with replica 2, sync with replica 1 and check we get expected tasks,
/// clear replica 1, sync again and check we get no tasks.
fn test_server_process_clear_actually_clears() {
    let mut db = open_test_db();
    let mut tx = db.transaction().unwrap();

    let replica1 = example_replica_1();
    let replica2 = example_replica_2();
    let ops = vec![example_add_uset_op_1(), example_add_uset_op_2()];

    process_sync(&mut tx, replica1, &[]).expect("failed to process sync");
    process_sync(&mut tx, replica2, &ops).expect("failed to process sync");

    let resp_ops = process_sync(&mut tx, replica1, &[]).expect("failed to process sync");
    assert_eq!(resp_ops, ops);

    let res = process_clear(&mut tx, replica1);
    assert!(res.is_ok(), "Failed to clear ops for replica 1 after syncing: {}", res.unwrap_err());

    let resp_ops = process_sync(&mut tx, replica1, &[]).expect("failed to process sync");
    assert_eq!(resp_ops, []);
}

#[test]
/// Check that clearing for an arbitrary replica, registered or not, does not clear existing tasks
/// in the database.
fn test_server_process_clear_existing_task() {
    let mut db = open_test_db();
    let mut tx = db.transaction().unwrap();

    let replica1 = example_replica_1();
    let replica2 = example_replica_2();

    let task = example_task_1();
    tx.add_task(&task).expect("Failed to add task");

    // register replica 1
    process_sync(&mut tx, replica1, &[]).unwrap();

    // clear on replica 1 already registered
    let res = process_clear(&mut tx, replica1);
    assert!(res.is_ok(), "Failed to clear replica 1: {}", res.unwrap_err());

    // clear on replica 2 not yet registered
    let res = process_clear(&mut tx, replica2);
    assert!(res.is_ok(), "Failed to clear replica 2: {}", res.unwrap_err());

    let tasks = tx.fetch_all_tasks().unwrap();
    assert_eq!(tasks, vec![task.clone(),]);

    // sync and register replica 2, get task
    let res = process_sync(&mut tx, replica2, &[]);
    assert!(res.is_ok(), "Failed to process replica 2 during initial sync after clear: {}", res.unwrap_err());

    let resp_ops = res.unwrap();
    assert_eq!(resp_ops, vec![USetOp::Add(task.clone()),]);
}

#[test]
/// Register 3 clients, send a task from 1, then clear from 2 and upon sync check we don't have anything for
/// 2, but when syncing without clearing there is a task for 3.
fn test_server_process_clear_three_clients_clear_vs_no_clear() {
    let mut db = open_test_db();
    let mut tx = db.transaction().unwrap();

    let replica1 = example_replica_1();
    let replica2 = example_replica_2();
    let replica3 = example_replica_3();

    let op = example_add_uset_op_2();

    // register all three
    process_sync(&mut tx, replica1, &[]).unwrap();
    process_sync(&mut tx, replica2, &[]).unwrap();
    process_sync(&mut tx, replica3, &[]).unwrap();

    // sync task with 1

    process_sync(&mut tx, replica1, &[op.clone()]).unwrap();

    // clear 2
    let res = process_clear(&mut tx, replica2);
    assert!(res.is_ok(), "Error while clearing replica 2: {}", res.unwrap_err());

    // sync 2, get nothing because we cleared
    let res = process_sync(&mut tx, replica2, &[]);
    assert!(res.is_ok(), "Error while syncing replica 2 after clearing: {}", res.unwrap_err());

    let resp_ops = res.unwrap();
    assert!(resp_ops.is_empty(), "Recieved operations after clearing from replica 2: {:?}", resp_ops);

    // sync 3 without clearing, get op from replica 1 sync
    let res = process_sync(&mut tx, replica3, &[]);
    assert!(res.is_ok(), "Error while syncing replica 3 without clearing: {}", res.unwrap_err());

    let resp_ops = res.unwrap();
    assert_eq!(resp_ops, vec![op.clone(),]);
}
