use crate::db::DBBackend;

use crate::db::tests::open_test_db;

use crate::task::test_utils::{example_task_1, example_task_break_1};

use crate::selection::{WeightedRandom, Top};

#[test]
fn test_db_skip_empty() {
    let mut selector = WeightedRandom::new(0.0);

    let mut db = open_test_db();
    let tx = db.transaction().expect("Failed to begin transaction");

    let res = tx.skip_current_task(&mut selector);
    assert!(res.is_ok(), "Error skipping current task: {}", res.unwrap_err());
}

#[test]
/// Add one task, skip it, check no current task is set and original task is still in db
fn test_db_skip_1() {
    let mut selector = Top::new();

    let mut db = open_test_db();
    let tx = db.transaction().expect("Failed to begin transaction");

    tx.add_task(&example_task_1()).expect("Adding task failed");
    tx.select_current_task(&mut selector).expect("Selecting task failed");

    let res = tx.skip_current_task(&mut selector);
    assert!(res.is_ok(), "Error skipping current task: {}", res.unwrap_err());

    // check no current task set
    let current_task_opt = tx.fetch_current_task().expect("Error fetching current task");
    assert!(current_task_opt.is_none(), "Current task is set even after skipping last one: {:?}", current_task_opt.unwrap());

    // check task list is not empty
    let tasks = tx.fetch_all_tasks().expect("Error fetching tasks");
    assert_eq!(1, tasks.len());
}

#[test]
/// Add two tasks, skip current, check that remaining task is the current task and both tasks are
/// in db
fn test_db_skip_2() {
    let mut selector = Top::new();

    let mut db = open_test_db();
    let tx = db.transaction().expect("Failed to begin transaction");

    tx.add_task(&example_task_break_1()).expect("Adding task failed");
    tx.add_task(&example_task_1()).expect("Adding task failed");
    tx.select_current_task(&mut selector).expect("Selecting task failed");

    let res = tx.skip_current_task(&mut selector);
    assert!(res.is_ok(), "Error skipping current task: {}", res.unwrap_err());

    // since we used top to select the initial current task and then skip it,
    // check task is set to the remaining Break category task
    let current_task = tx.fetch_current_task()
        .expect("Error fetching current task").expect("No current task was set");
    assert_eq!(example_task_break_1(), current_task);

    // check task list contains both tasks
    let tasks = tx.fetch_all_tasks().expect("Error fetching tasks");
    assert_eq!(2, tasks.len(), "Incorrect number of tasks in db after skipping task: {:?}", tasks);


    // now skip again and check current gets set to the original Task 

    let res = tx.skip_current_task(&mut selector);
    assert!(res.is_ok(), "Error skipping current task: {}", res.unwrap_err());

    let current_task = tx.fetch_current_task()
        .expect("Error fetching current task").expect("No current task was set");
    assert_eq!(example_task_1(), current_task);

    // check task list contains both tasks
    let tasks = tx.fetch_all_tasks().expect("Error fetching tasks");
    assert_eq!(2, tasks.len(), "Incorrect number of tasks in db after skipping task: {:?}", tasks);
}

// FIXME: I couldn't really think of a good proptest for this.
