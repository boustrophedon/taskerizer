use std::collections::HashSet;

use uuid::Uuid;

use crate::db::DBBackend;
use crate::selection::{Top, WeightedRandom};
use crate::sync::USetOp;

use crate::db::tests::open_test_db;
use crate::task::test_utils::{example_task_1, example_task_2};
use super::uset_add_list_arb;


// there isn't really much difference between these two tests because USetOp::Add is literally a
// wrapper around DBBackend::add_task but it's good for refactoring checks etc

// additionally, the other tests below are exactly the same tests as in db::tests::remove_by_uuid
// but if we ever change semantics of the uset op, the tests will diverge, so it's better to have
// them now.

#[test]
/// Add via db op, remove via uset
fn test_uset_remove_task_local_1() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let task = example_task_1();
    tx.add_task(&task).expect("failed to add task");

    let op = USetOp::Remove(task.uuid().clone());
    let res = op.apply_to_db(&tx);
    assert!(res.is_ok(), "Could not remove task from database via USetOp: {}", res.unwrap_err());

    let db_tasks = tx.fetch_all_tasks().expect("Couldn't get tasks");
    assert!(db_tasks.is_empty(), "DB contained task after removing only task: {:?}", db_tasks);
}

#[test]
/// Add via uset, remove via uset
fn test_uset_remove_task_remote_1() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let task = example_task_1();

    // add example task
    let add_op = USetOp::Add(task.clone());
    add_op.apply_to_db(&tx).expect("Failed to add task via uset");

    let remove_op = USetOp::Remove(task.uuid().clone());
    let res = remove_op.apply_to_db(&tx);
    assert!(res.is_ok(), "Could not remove task from database via USetOp: {}", res.unwrap_err());

    let report = res.unwrap();
    assert!(report.is_empty(), "Report contained something even though we didn't remove current task: {:?}", report);

    let db_tasks = tx.fetch_all_tasks().expect("Couldn't get tasks");
    assert!(db_tasks.is_empty(), "DB contained task after removing only task: {:?}", db_tasks);
}

#[test]
/// Add task, set current, remove via uset, check we get current task back.
/// Use apply_remove_to_db directly.
fn test_uset_remove_current() {
    let mut selector = Top::new();
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let task = example_task_1();
    tx.add_task(&task).expect("Failed removing task");
    tx.select_current_task(&mut selector).expect("Failed choosing current task");

    let res = USetOp::apply_remove_to_db(&tx, task.uuid());
    assert!(res.is_ok(), "Failed removing task via USetOp: {}", res.unwrap_err());

    let opt = res.unwrap();
    assert!(opt.is_some(), "No task returned from remove op even though we set current");
    
    let removed_current = opt.unwrap();
    assert_eq!(removed_current, task);
}

#[test]
/// Add task, set current, add other task, remove other task.
fn test_uset_remove_non_current() {
    let mut selector = Top::new();
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let task1 = example_task_1();
    tx.add_task(&task1).expect("Failed adding task");
    tx.select_current_task(&mut selector).expect("Failed choosing current task");

    let task2 = example_task_2();
    tx.add_task(&task2).expect("Failed adding task");

    let res = USetOp::apply_remove_to_db(&tx, task2.uuid());
    assert!(res.is_ok(), "Failed removing task via USetOp: {}", res.unwrap_err());

    let opt = res.unwrap();
    assert!(opt.is_none(), "Task returned from remove op even though we didn't remove current: {:?}", opt.unwrap());
}

proptest! {
    /// Add tasks, then remove them, checking other tasks are still in db. Then try applying
    /// removes again and check we get no error.
    #[test]
    fn test_uset_remove_arb(add_ops in uset_add_list_arb()) {
        let mut db = open_test_db();
        let tx = db.transaction().unwrap();

        for op in &add_ops {
            op.apply_to_db(&tx).expect("Failed applying add op to db");
        }

        let remove_ops: Vec<USetOp> = add_ops.into_iter().map(|op| op.into_remove()).collect();

        // get uuids from remove ops
        let mut uuids: HashSet<Uuid> = remove_ops.iter().cloned().map(|op| op.unwrap_remove()).collect();

        for op in &remove_ops {
            let remove_uuid = op.clone().unwrap_remove();
            let res = USetOp::apply_remove_to_db(&tx, &remove_uuid);
            uuids.remove(&remove_uuid);
            prop_assert!(res.is_ok(), "Error removing task via USetOp: {}", res.unwrap_err());

            let opt = res.unwrap();
            prop_assert!(opt.is_none(), "Removed current task even though we didn't set one");

            let db_tasks = tx.fetch_all_tasks().expect("Couldn't fetch tasks");
            prop_assert_eq!(db_tasks.len(), uuids.len());

            for task in &db_tasks {
                prop_assert!(uuids.contains(task.uuid()), "Extra task found in db that wasn't removed properly: {:?}", &task);
            }
        }

        for op in &remove_ops {
            let remove_uuid = op.clone().unwrap_remove();
            let res = USetOp::apply_remove_to_db(&tx, &remove_uuid);
            prop_assert!(res.is_ok(), "Error removing previously-removed task via USetOp: {}", res.unwrap_err());

            let opt = res.unwrap();
            prop_assert!(opt.is_none(), "Removed current task even though db is empty");

            let db_tasks = tx.fetch_all_tasks().expect("Couldn't fetch tasks");
            prop_assert_eq!(db_tasks.len(), 0);
        }
    }
}

proptest! {
    /// Same as above, but set the current when removing tasks
    #[test]
    fn test_uset_remove_current_arb(add_ops in uset_add_list_arb()) {
        let mut selector = WeightedRandom::new(0.5);
        let mut db = open_test_db();
        let tx = db.transaction().unwrap();

        for op in &add_ops {
            op.apply_to_db(&tx).expect("Failed applying add op to db");
        }

        let remove_ops: Vec<USetOp> = add_ops.into_iter().map(|op| op.into_remove()).collect();

        // get uuids from remove ops
        let mut uuids: HashSet<Uuid> = remove_ops.iter().cloned().map(|op| op.unwrap_remove()).collect();

        // select current task, do remove operation. if we removed current task, check we actually
        // did.
        for op in &remove_ops {
            tx.select_current_task(&mut selector).expect("Failed to select current task");
            let db_current_task = tx.fetch_current_task().expect("Failed to fetch current task").unwrap();

            let remove_uuid = op.clone().unwrap_remove();
            let res = USetOp::apply_remove_to_db(&tx, &remove_uuid);
            uuids.remove(&remove_uuid);
            prop_assert!(res.is_ok(), "Error removing task via USetOp: {}", res.unwrap_err());

            let opt = res.unwrap();
            if let Some(removed_current_task) = opt {
                prop_assert_eq!(db_current_task, removed_current_task,
                           "Task was returned by remove op but not equal to current task");
            }

            let db_tasks = tx.fetch_all_tasks().expect("Couldn't fetch tasks");
            prop_assert_eq!(db_tasks.len(), uuids.len());

            for task in &db_tasks {
                prop_assert!(uuids.contains(task.uuid()), "Extra task found in db that wasn't removed properly: {:?}", &task);
            }
        }
    }
}

proptest! {
    /// Add tasks, then remove a bunch of non-existant tasks, check that no tasks are removed.
    #[test]
    fn test_uset_remove_non_existant_arb(add_ops in uset_add_list_arb(), remove_ops in uset_add_list_arb()) {
        let mut db = open_test_db();
        let tx = db.transaction().unwrap();

        for op in &add_ops {
            op.apply_to_db(&tx).expect("Failed applying add op to db");
        }

        let remove_ops: Vec<USetOp> = remove_ops.into_iter().map(|op| op.into_remove()).collect();

        for op in &remove_ops {
            let remove_uuid = op.clone().unwrap_remove();
            let res = USetOp::apply_remove_to_db(&tx, &remove_uuid);
            prop_assert!(res.is_ok(), "Error removing non-existant task via USetOp: {}", res.unwrap_err());

            let opt = res.unwrap();
            prop_assert!(opt.is_none(), "Removed current task even though we didn't remove any task");

            let db_tasks = tx.fetch_all_tasks().expect("Couldn't fetch tasks");
            prop_assert_eq!(db_tasks.len(), add_ops.len());
        }
    }
}
