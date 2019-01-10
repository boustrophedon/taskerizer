use crate::db::DBBackend;

use crate::db::tests::open_test_db;

use crate::task::test_utils::{example_task_1, example_task_break_1, arb_task_list};

use crate::selection::WeightedRandom;

#[test]
fn test_db_select_current_empty() {
    // break probability = 0.0
    let mut selector = WeightedRandom::new(0.0);

    let mut db = open_test_db();
    let tx = db.transaction().expect("Failed to begin transaction");

    let res = tx.select_current_task(&mut selector);
    assert!(res.is_ok(), "Error selecting new task with no tasks in db: {}", res.unwrap_err());
}

// test with break probability 0 and 1 with both breaks and tasks in db
// maybe do a test with p=0.5
// maybe test with Top here as well

#[test]
fn test_db_select_current_one_task() {
    // select_current_task should select the task because there is only one even though the break
    // probability = 1
    let mut selector = WeightedRandom::new(1.0);

    let mut db = open_test_db();
    let tx = db.transaction().expect("Failed to begin transaction");

    tx.add_task(&example_task_1()).expect("Adding task failed");

    let res = tx.select_current_task(&mut selector);
    assert!(res.is_ok(), "Choosing current task with one task failed: {}", res.unwrap_err());
}

#[test]
fn test_db_select_current_one_break() {
    // select_current_task should select the break because there is only one even though break
    // probability = 0
    let mut selector = WeightedRandom::new(0.0);

    let mut db = open_test_db();
    let tx = db.transaction().expect("Failed to begin transaction");

    tx.add_task(&example_task_break_1()).expect("Adding task failed");

    let res = tx.select_current_task(&mut selector);
    assert!(res.is_ok(), "Choosing current task with one break failed: {}", res.unwrap_err());
}

proptest! {
    #[test]
    fn test_db_select_current_arb(tasks in arb_task_list()) {
        // break probability = 0
        let mut selector_task = WeightedRandom::new(0.0);
        // break probability = 1
        let mut selector_break = WeightedRandom::new(1.0);

        let mut db = open_test_db();
        let tx = db.transaction().expect("Failed to begin transaction");

        // keep track of which categories of tasks we have
        let mut has_break = false;
        let mut has_task = false;

        for task in &tasks {
            if task.is_break() {
                has_break = true;
            } else {
                has_task = true;
            }
            tx.add_task(task).expect("adding task failed");
        }

        // select current task with break probability = 0,
        // then check that if we have a Task category task, it was chosen since we can't choose
        // breaks. Otherwise, make sure that we chose a Break category task
        let res = tx.select_current_task(&mut selector_task);
        prop_assert!(res.is_ok(), "Selecting current task using task-selector failed: {}", res.unwrap_err());

        let opt = tx.fetch_current_task().expect("Error getting current task after setting it");
        prop_assert!(opt.is_some(), "No current task set even though we just selected one.");
        let current = opt.unwrap();

        if has_task {
            prop_assert!(!current.is_break(),
                "Current task is not Task category even though there are Tasks in the db: {:?}", current);
        }
        else {
            prop_assert!(current.is_break(),
                "Current task is not Break category even though there are no Tasks in the db: {:?}", current);
        }


        // same as above, with break/task flipped
        // select current task with break probability = 1,
        // then check that if we have a Break category task, it was chosen since we can't choose
        // Tasks. Otherwise, make sure that we chose a Task category task
        let res = tx.select_current_task(&mut selector_break);
        prop_assert!(res.is_ok(), "Selecting current task using break-selector failed: {}", res.unwrap_err());

        let opt = tx.fetch_current_task().expect("Error getting current task after setting it");
        prop_assert!(opt.is_some(), "No current task set even though we just selected one.");
        let current = opt.unwrap();

        if has_break {
            prop_assert!(current.is_break(),
                "Current task is not Break category even though there are Breaks in the db: {:?}", current);
        }
        else {
            prop_assert!(!current.is_break(),
                "Current task is not Task category even though there are no Breaks in the db: {:?}", current);
        }
    }
}
