use crate::db::DBBackend;

use crate::db::tests::open_test_db;

use crate::task::test_utils::{example_task_1, example_task_break_1, arb_task_list};

use crate::selection::Top;

#[test]
fn test_db_complete_empty() {
    let mut db = open_test_db();
    let tx = db.transaction().expect("Failed to begin transaction");

    let res = tx.complete_current_task();
    assert!(res.is_ok(), "Error completing current task: {}", res.unwrap_err());

    let opt = res.unwrap();
    assert!(opt.is_none(), "Complete returned a task with nothing in the db: {:?}", opt.unwrap());
}

#[test]
/// Add a task, select it, check when we complete the current task it returns the same one and
/// there is no current task.
fn test_db_complete_1() {
    let mut selector = Top::new();

    let mut db = open_test_db();
    let tx = db.transaction().expect("Failed to begin transaction");

    tx.add_task(&example_task_1()).expect("Adding task failed");
    tx.select_current_task(&mut selector).expect("Selecting task failed");

    // check task is set to the only one possible
    let res = tx.complete_current_task();
    assert!(res.is_ok(), "Error completing current task: {}", res.unwrap_err());

    let opt = res.unwrap();
    assert!(opt.is_some(), "No task returned after complete operation.");

    let task = opt.unwrap();
    assert_eq!(task, example_task_1());

    // check no current task set
    let current_task_opt = tx.fetch_current_task().expect("Error fetching current task");
    assert!(current_task_opt.is_none(), "Current task is set even after completing last one: {:?}", current_task_opt.unwrap());

    // check task list is empty
    let tasks = tx.fetch_all_tasks().expect("Error fetching tasks");
    assert!(tasks.is_empty(), "Tasks in db after completing only remaining task: {:?}", tasks);
}

#[test]
/// Add a task and a break, select the task (using Top), check when we complete the current task it
/// returns the Task. Then do the same for the remaining Break.
fn test_db_complete_2() {
    let mut selector = Top::new();

    let mut db = open_test_db();
    let tx = db.transaction().expect("Failed to begin transaction");

    tx.add_task(&example_task_break_1()).expect("Adding task failed");
    tx.add_task(&example_task_1()).expect("Adding task failed");
    tx.select_current_task(&mut selector).expect("Selecting task failed");

    // check task is set to the Task category task, since we used Top selection strategy
    let current_task = tx.fetch_current_task()
        .expect("Error fetching current task").expect("No current task was set");
    let res = tx.complete_current_task();
    assert!(res.is_ok(), "Error completing current task: {}", res.unwrap_err());

    let opt = res.unwrap();
    assert!(opt.is_some(), "No task returned after complete operation.");

    let task = opt.unwrap();
    assert_eq!(task, example_task_1());
    assert_eq!(task, current_task);

    // check task list contains remaining Break
    let tasks = tx.fetch_all_tasks().expect("Error fetching tasks");
    assert_eq!(1, tasks.len(), "Incorrect number of tasks in db after completing task: {:?}", tasks);
    assert_eq!(tasks[0], example_task_break_1());

    // check no current task set
    let current_task_opt = tx.fetch_current_task().expect("Error fetching current task");
    assert!(current_task_opt.is_none(), "Current task is set even after completing last one: {:?}", current_task_opt.unwrap());


    // do the same as above with the remaining Break 
    tx.select_current_task(&mut selector).expect("Selecting task failed");
    let current_task = tx.fetch_current_task()
        .expect("Error fetching current task").expect("No current task was set");

    let res = tx.complete_current_task();
    assert!(res.is_ok(), "Error completing current task: {}", res.unwrap_err());

    let opt = res.unwrap();
    assert!(opt.is_some(), "No task returned after complete operation.");

    let task = opt.unwrap();
    assert_eq!(task, example_task_break_1());
    assert_eq!(task, current_task);

    // check no current task set
    let current_task_opt = tx.fetch_current_task().expect("Error fetching current task");
    assert!(current_task_opt.is_none(), "Current task is set even after completing last one: {:?}", current_task_opt.unwrap());

    // check task list is empty
    let tasks = tx.fetch_all_tasks().expect("Error fetching tasks");
    assert!(tasks.is_empty(), "Tasks in db after completing only remaining task: {:?}", tasks);
}

proptest! {
    #[test]
    /// complete current task each loop and check that the previously selected one is the same
    /// as the newly completed one. then check there is one fewer task in the task list.
    fn test_db_complete_arb(tasks in arb_task_list()) {
        let mut selector = Top::new();

        let mut db = open_test_db();
        let tx = db.transaction().expect("Failed to begin transaction");

        for task in &tasks {
            tx.add_task(task).expect("Failed to add task");
        }


        let mut db_tasks = tx.fetch_all_tasks().expect("Failed to get db tasks");
        prop_assert_eq!(db_tasks.len(), tasks.len());
        while !db_tasks.is_empty() {
            tx.select_current_task(&mut selector).expect("Failed to select current task");
            let current_task = tx.fetch_current_task()
                .expect("Error fetching current task").expect("No current task was set");

            let res = tx.complete_current_task();
            prop_assert!(res.is_ok(), "Error completing current task: {}", res.unwrap_err());

            let opt = res.unwrap();
            prop_assert!(opt.is_some(), "No task returned after complete operation.");

            let task = opt.unwrap();
            prop_assert_eq!(task, current_task);

            let remaining_tasks = tx.fetch_all_tasks().expect("Failed to get db tasks");
            prop_assert_eq!(remaining_tasks.len(), db_tasks.len()-1);

            // check no current task set
            let current_task_opt = tx.fetch_current_task().expect("Error fetching current task");
            prop_assert!(current_task_opt.is_none(), "Current task is set even after completing last one: {:?}", current_task_opt.unwrap());

            db_tasks = remaining_tasks;
        }

        // check no current task set
        let current_task_opt = tx.fetch_current_task().expect("Error fetching current task");
        prop_assert!(current_task_opt.is_none(), "Current task is set even after completing last one: {:?}", current_task_opt.unwrap());

        // check task list is empty
        let tasks = tx.fetch_all_tasks().expect("Error fetching tasks");
        prop_assert!(tasks.is_empty(), "Tasks in db after completing only remaining task: {:?}", tasks);

    }
}
