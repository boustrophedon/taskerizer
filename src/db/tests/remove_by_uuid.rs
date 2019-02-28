use std::collections::HashSet;

use crate::db::DBBackend;
use crate::selection::{Top, WeightedRandom};

use crate::db::tests::open_test_db;
use crate::task::test_utils::{example_task_1, example_task_2, arb_task_list};
use crate::sync::ReplicaUuid;

#[test]
/// Add task, set current, remove via uuuid, check we get current task back.
fn test_db_remove_current() {
    let mut selector = Top::new();
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let task = example_task_1();
    tx.add_task(&task).expect("Failed removing task");
    tx.select_current_task(&mut selector).expect("Failed choosing current task");

    let res = DBBackend::remove_task_by_uuid(&tx, task.uuid());
    assert!(res.is_ok(), "Failed removing task by uuid: {}", res.unwrap_err());

    let opt = res.unwrap();
    assert!(opt.is_some(), "No task returned from remove_by_uuid even though we set current");
    
    let removed_current = opt.unwrap();
    assert_eq!(removed_current, task);
}

#[test]
/// Add task, set current, add other task, remove other task.
fn test_db_remove_uuid_non_current() {
    let mut selector = Top::new();
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let task1 = example_task_1();
    tx.add_task(&task1).expect("Failed adding task");
    tx.select_current_task(&mut selector).expect("Failed choosing current task");

    let task2 = example_task_2();
    tx.add_task(&task2).expect("Failed adding task");

    let res = DBBackend::remove_task_by_uuid(&tx, task2.uuid());
    assert!(res.is_ok(), "Failed removing task by uuid: {}", res.unwrap_err());

    let opt = res.unwrap();
    assert!(opt.is_none(), "Task returned from remove_by_uuid even though we didn't remove current: {:?}", opt.unwrap());

    let db_tasks = tx.fetch_all_tasks().expect("Failed to fetch tasks");
    assert_eq!(db_tasks.len(), 1);
}

proptest! {
    /// Add tasks, then remove them, checking other tasks are still in db. Then try removing the
    /// same uuids again and check we get no error.
    #[test]
    fn test_db_remove_uuid_arb(tasks in arb_task_list()) {
        let mut db = open_test_db();
        let tx = db.transaction().unwrap();

        for task in &tasks {
            tx.add_task(task).expect("Failed applying add op to db");
        }

        // get uuids from tasks
        let mut uuids: HashSet<ReplicaUuid> = tasks.iter().map(|t| t.uuid().clone()).collect();

        for task in &tasks {
            let remove_uuid = task.uuid();
            let res = DBBackend::remove_task_by_uuid(&tx, &remove_uuid);
            uuids.remove(&remove_uuid);
            prop_assert!(res.is_ok(), "Error removing task by uuid: {}", res.unwrap_err());

            let opt = res.unwrap();
            prop_assert!(opt.is_none(), "Removed current task even though we didn't set one");

            let db_tasks = tx.fetch_all_tasks().expect("Couldn't fetch tasks");
            prop_assert_eq!(db_tasks.len(), uuids.len());

            for db_task in &db_tasks {
                prop_assert!(uuids.contains(db_task.uuid()), "Extra task found in db that wasn't removed properly: {:?}", &task);
            }
        }

        for task in &tasks {
            let remove_uuid = task.uuid();
            let res = DBBackend::remove_task_by_uuid(&tx, &remove_uuid);
            prop_assert!(res.is_ok(), "Error removing previously-removed task uuid: {}", res.unwrap_err());

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
    fn test_db_remove_uuid_current_arb(tasks in arb_task_list()) {
        let mut selector = WeightedRandom::new(0.5);
        let mut db = open_test_db();
        let tx = db.transaction().unwrap();

        for task in &tasks {
            tx.add_task(task).expect("Failed adding task to db");
        }

        // get uuids from tasks
        let mut uuids: HashSet<ReplicaUuid> = tasks.iter().map(|t| t.uuid().clone()).collect();

        // select current task, do remove operation. if we removed current task, check we actually
        // did.
        for task in &tasks {
            tx.select_current_task(&mut selector).expect("Failed to select current task");
            let db_current_task = tx.fetch_current_task().expect("Failed to fetch current task").unwrap();

            let remove_uuid = task.uuid();
            let res = DBBackend::remove_task_by_uuid(&tx, &remove_uuid);
            uuids.remove(&remove_uuid);
            prop_assert!(res.is_ok(), "Error removing task by uuid: {}", res.unwrap_err());

            let opt = res.unwrap();
            if let Some(removed_current_task) = opt {
                prop_assert_eq!(db_current_task, removed_current_task,
                           "Task was returned by remove_by_uuid but not equal to current task");
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
    fn test_db_remove_uuid_non_existant_arb(add_tasks in arb_task_list(), remove_tasks in arb_task_list()) {
        let mut db = open_test_db();
        let tx = db.transaction().unwrap();

        for task in &add_tasks {
            tx.add_task(task).expect("Failed adding task to db");
        }

        for task in &remove_tasks {
            let remove_uuid = task.uuid();
            let res = DBBackend::remove_task_by_uuid(&tx, &remove_uuid);
            prop_assert!(res.is_ok(), "Error removing non-existant task by uuid: {}", res.unwrap_err());

            let opt = res.unwrap();
            prop_assert!(opt.is_none(), "Removed current task even though we didn't remove task in db");

            let db_tasks = tx.fetch_all_tasks().expect("Couldn't fetch tasks");
            prop_assert_eq!(db_tasks.len(), add_tasks.len());
        }
    }
}
