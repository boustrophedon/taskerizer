use crate::db::DBBackend;

use crate::db::tests::open_test_db;

use crate::task::Task;
use crate::task::test_utils::{example_task_list, arb_task_list};

#[test]
fn test_db_list_empty() {
    let mut db = open_test_db();

    // get nothing from db
    let res = db.get_all_tasks();
    assert!(res.is_ok(), "Tasks could not be retrieved: {:?}", res.unwrap_err());
    let db_tasks = res.unwrap();

    // check nothing was returned
    assert_eq!(db_tasks.len(), 0);
}

#[test]
fn test_db_list_invalid_task_empty() {
    let mut db = open_test_db();

    let task = Task::example_invalid_empty_desc();
    db.add_task(&task).expect("Adding task failed");

    let res = db.get_all_tasks();
    assert!(res.is_err(), "No error when trying to deserialize invalid task: {:?}", res);

    let err = res.unwrap_err();
    assert!(err.to_string().contains("Empty task description"), "Reading invalid task has incorrect error message: {}", err);
}

#[test]
fn test_db_list_invalid_task_zero_priority() {
    let mut db = open_test_db();

    let task = Task::example_invalid_zero_priority();
    db.add_task(&task).expect("Adding task failed");

    let res = db.get_all_tasks();
    assert!(res.is_err(), "No error when trying to deserialize invalid task: {:?}", res);

    let err = res.unwrap_err();
    assert!(err.to_string().contains("Zero priority"), "Reading invalid task has incorrect error message: {}", err);
}


#[test]
fn test_db_list_added_manually() {
    let mut db = open_test_db();

    let tasks = example_task_list();

    // add all tasks to db
    for task in &tasks {
        let res = db.add_task(&task);
        assert!(res.is_ok(), "Adding task failed. task: {:?}, err: {}", task, res.unwrap_err());
    }

    // get tasks back from db
    let res = db.get_all_tasks();
    assert!(res.is_ok(), "Tasks could not be retrieved: {:?}", res.unwrap_err());
    let db_tasks = res.unwrap();

    // check number of tasks returned is correct
    assert_eq!(db_tasks.len(), tasks.len());

    // check every task made it back
    for task in &tasks {
        assert!(db_tasks.contains(task), "tasks returned from db does not contain task {:?}", task);
    }

}

proptest! {
    #[test]
    fn test_db_list_arb(tasks in arb_task_list()) {
        let mut db = open_test_db();

        // add all tasks to db
        for task in &tasks {
            let res = db.add_task(&task);
            prop_assert!(res.is_ok(), "Adding task failed. task: {:?}, err: {}", task, res.unwrap_err());
        }

        // get tasks back
        let res = db.get_all_tasks();
        prop_assert!(res.is_ok(), "Tasks could not be retrieved: {:?}", res.unwrap_err());
        let db_tasks = res.unwrap();

        // check number of tasks returned is correct
        prop_assert_eq!(db_tasks.len(), tasks.len());

        // check every task made it back
        for task in &tasks {
            prop_assert!(db_tasks.contains(task), "tasks returned from db does not contain task {:?}", task);
        }
    }
}
