use crate::db::DBTransaction;
use crate::db::tests::open_test_db;

use crate::task::Task;
use crate::task::test_utils::{example_task_1, example_task_break_1, arb_task_list};

// TODO these tests are too long and could be broken up into multiple smaller tests.

#[test]
fn test_tx_list_task_rollback() {
    // Outline: add task, check it's there, rollback, check it's not there
    //

    let task = example_task_1();
    let reward = example_task_break_1();

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();
    tx.add_task(&task).unwrap();
    tx.add_task(&reward).unwrap();

    // verify task
    let res = tx.fetch_tasks();
    assert!(res.is_ok(), "Getting tasks failed: {}", res.unwrap_err());

    let tasks = res.unwrap();
    assert!(tasks.len() == 1, "Not enough tasks expected 1 got {}", tasks.len());
    assert!(tasks[0].1 == task, "Task retrieved from database was incorrect, expected {:?} got {:?}", task, tasks[0].1);

    // verify break
    let res = tx.fetch_breaks();
    assert!(res.is_ok(), "Getting breaks failed: {}", res.unwrap_err());

    let breaks = res.unwrap();
    assert!(breaks.len() == 1, "Not enough breaks expected 1 got {}", breaks.len());
    assert!(breaks[0].1 == reward, "Break task retrieved from database was incorrect, expected {:?} got {:?}", reward, breaks[0].1);

    tx.rollback().unwrap();

    // verify that there are no tasks in the db

    let tx = db.transaction().unwrap();
    // verify no tasks
    let res = tx.fetch_tasks();
    assert!(res.is_ok(), "Getting tasks failed: {}", res.unwrap_err());

    let tasks = res.unwrap();
    assert!(tasks.len() == 0, "Tasks still in db: tasks expected 0 got {}", tasks.len());

    // verify no breaks
    let res = tx.fetch_breaks();
    assert!(res.is_ok(), "Getting breaks failed: {}", res.unwrap_err());

    let breaks = res.unwrap();
    assert!(breaks.len() == 0, "Breaks still in db: expected 0 got {}", tasks.len());
}

#[test]
fn test_tx_list_task_commit() {
    // Outline: add task, check it's there, commit, check it's still there
    //

    let task = example_task_1();
    let reward = example_task_break_1();

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();
    tx.add_task(&task).unwrap();
    tx.add_task(&reward).unwrap();

    // verify task
    let res = tx.fetch_tasks();
    assert!(res.is_ok(), "Getting tasks failed: {}", res.unwrap_err());

    let tasks = res.unwrap();
    assert!(tasks.len() == 1, "Not enough tasks expected 1 got {}", tasks.len());
    assert!(tasks[0].1 == task, "Task retrieved from database was incorrect, expected {:?} got {:?}", task, tasks[0].1);

    // verify break
    let res = tx.fetch_breaks();
    assert!(res.is_ok(), "Getting breaks failed: {}", res.unwrap_err());

    let breaks = res.unwrap();
    assert!(breaks.len() == 1, "Not enough breaks expected 1 got {}", breaks.len());
    assert!(breaks[0].1 == reward, "Break task retrieved from database was incorrect, expected {:?} got {:?}", reward, breaks[0].1);

    tx.commit().unwrap();

    // verify tasks still in the db
    // same exact checks as above

    let tx = db.transaction().unwrap();
    // verify task
    let res = tx.fetch_tasks();
    assert!(res.is_ok(), "Getting tasks failed: {}", res.unwrap_err());

    let tasks = res.unwrap();
    assert!(tasks.len() == 1, "Not enough tasks expected 1 got {}", tasks.len());
    assert!(tasks[0].1 == task, "Task retrieved from database was incorrect, expected {:?} got {:?}", task, tasks[0].1);

    // verify break
    let res = tx.fetch_breaks();
    assert!(res.is_ok(), "Getting breaks failed: {}", res.unwrap_err());

    let breaks = res.unwrap();
    assert!(breaks.len() == 1, "Not enough tasks expected 1 got {}", tasks.len());
    assert!(breaks[0].1 == reward, "Break task retrieved from database was incorrect, expected {:?} got {:?}", reward, tasks[0].1);
}

proptest! {
    #[test]
    fn test_tx_add_task_rollback_arb(all_tasks in arb_task_list()) {
        let mut db = open_test_db();
        let tx = db.transaction().unwrap();

        // add tasks
        for task in &all_tasks {
            let res = tx.add_task(&task);
            prop_assert!(res.is_ok(), "Adding task failed: {}", res.unwrap_err());
        }

        // get tasks
        let res = tx.fetch_tasks();
        prop_assert!(res.is_ok(), "Getting tasks failed: {}", res.unwrap_err());
        let tasks: Vec<Task> = res.unwrap().into_iter().map(|t| t.1).collect();

        // get breaks
        let res = tx.fetch_breaks();
        prop_assert!(res.is_ok(), "Getting breaks failed: {}", res.unwrap_err());
        let breaks: Vec<Task> = res.unwrap().into_iter().map(|t| t.1).collect();

        // verify number of tasks
        let expected = all_tasks.len();
        let got = tasks.len() + breaks.len();
        prop_assert!(expected == got, "Number of tasks returned from db not the same as added: expected {} got {}", expected, got);

        // verify every task made it back
        for task in &all_tasks {
            if tasks.contains(task) {
                prop_assert!(!task.is_break(), "Task was in tasks list but was a break: {:?}", task);
                prop_assert!(!breaks.contains(task), "Task was a task but was in breaks list as well: {:?}", task);
            }
            else if breaks.contains(task) {
                prop_assert!(task.is_break(), "Task was in breaks but was not a break: {:?}", task);
                prop_assert!(!tasks.contains(task), "Task was a break but was in tasks list as well: {:?}", task);
            }
            else {
                prop_assert!(false, "Input task was not present in db: {:?}", task);
            }
        }

        tx.rollback().unwrap();

        let tx = db.transaction().unwrap();
        // get tasks
        let res = tx.fetch_tasks();
        prop_assert!(res.is_ok(), "Getting tasks failed: {}", res.unwrap_err());
        let tasks: Vec<Task> = res.unwrap().into_iter().map(|t| t.1).collect();

        // get breaks
        let res = tx.fetch_breaks();
        prop_assert!(res.is_ok(), "Getting breaks failed: {}", res.unwrap_err());
        let breaks: Vec<Task> = res.unwrap().into_iter().map(|t| t.1).collect();

        // verify no tasks
        let got = tasks.len() + breaks.len();
        prop_assert!(0 == got, "Got tasks even after rolling back: got {}", got);
    }
}

proptest! {
    #[test]
    fn test_tx_add_task_commit_arb(all_tasks in arb_task_list()) {
        let mut db = open_test_db();
        let tx = db.transaction().unwrap();

        // add tasks
        for task in &all_tasks {
            let res = tx.add_task(&task);
            prop_assert!(res.is_ok(), "Adding task failed: {}", res.unwrap_err());
        }

        // get tasks
        let res = tx.fetch_tasks();
        prop_assert!(res.is_ok(), "Getting tasks failed: {}", res.unwrap_err());
        let tasks: Vec<Task> = res.unwrap().into_iter().map(|t| t.1).collect();

        // get breaks
        let res = tx.fetch_breaks();
        prop_assert!(res.is_ok(), "Getting breaks failed: {}", res.unwrap_err());
        let breaks: Vec<Task> = res.unwrap().into_iter().map(|(_,t)| t).collect();

        // verify number of tasks
        let expected = all_tasks.len();
        let got = tasks.len() + breaks.len();
        prop_assert!(expected == got, "Number of tasks returned from db not the same as added: expected {} got {}", expected, got);

        // verify every task made it back
        for task in &all_tasks {
            if tasks.contains(task) {
                prop_assert!(!task.is_break(), "Task was in tasks list but was a break: {:?}", task);
                prop_assert!(!breaks.contains(task), "Task was a task but was in breaks list as well: {:?}", task);
            }
            else if breaks.contains(task) {
                prop_assert!(task.is_break(), "Task was in breaks but was not a break: {:?}", task);
                prop_assert!(!tasks.contains(task), "Task was a break but was in tasks list as well: {:?}", task);
            }
            else {
                prop_assert!(false, "Input task was not present in db: {:?}", task);
            }
        }

        tx.commit().unwrap();

        let tx = db.transaction().unwrap();
        // get tasks
        let res = tx.fetch_tasks();
        prop_assert!(res.is_ok(), "Getting tasks failed: {}", res.unwrap_err());
        let tasks: Vec<Task> = res.unwrap().into_iter().map(|t| t.1).collect();

        // get breaks
        let res = tx.fetch_breaks();
        prop_assert!(res.is_ok(), "Getting breaks failed: {}", res.unwrap_err());
        let breaks: Vec<Task> = res.unwrap().into_iter().map(|t| t.1).collect();

        // same verifications as above
        // verify number of tasks
        let expected = all_tasks.len();
        let got = tasks.len() + breaks.len();
        prop_assert!(expected == got, "Number of tasks returned from db not the same as added: expected {} got {}", expected, got);

        // verify every task made it back
        for task in &all_tasks {
            if tasks.contains(task) {
                prop_assert!(!task.is_break(), "Task was in tasks list but was a break: {:?}", task);
                prop_assert!(!breaks.contains(task), "Task was a task but was in breaks list as well: {:?}", task);
            }
            else if breaks.contains(task) {
                prop_assert!(task.is_break(), "Task was in breaks but was not a break: {:?}", task);
                prop_assert!(!tasks.contains(task), "Task was a break but was in tasks list as well: {:?}", task);
            }
            else {
                prop_assert!(false, "Input task was not present in db: {:?}", task);
            }
        }
    }
}
