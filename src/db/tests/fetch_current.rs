use crate::db::DBBackend;

use crate::db::tests::open_test_db;

use crate::task::Task;
use crate::task::test_utils::{example_task_1, example_task_break_1, example_task_2, arb_task_list};

use crate::selection::{Top, WeightedRandom};


#[test]
fn test_db_fetch_current_no_tasks() {
    let mut db = open_test_db();
    let tx = db.transaction().expect("Failed to begin transaction");

    let res = tx.fetch_current_task();
    assert!(res.is_ok(), "Getting current task failed: {}", res.unwrap_err());

    let no_task = res.unwrap();
    assert!(no_task.is_none(), "Without adding a task, somehow there was a current task: {:?}", no_task);
}

#[test]
fn test_db_fetch_current_one_task() {
    let mut selector = Top::new();

    let mut db = open_test_db();
    let tx = db.transaction().expect("Failed to begin transaction");
    
    tx.add_task(&example_task_1()).expect("Failed adding task to db in test");

    tx.select_current_task(&mut selector).expect("Failed to select current task");

    let res = tx.fetch_current_task();
    assert!(res.is_ok(), "Error fetching the current task: {}", res.unwrap_err());

    let opt = res.unwrap();
    assert!(opt.is_some(), "No current task after selecting one");

    let task = opt.unwrap();
    assert_eq!(task, example_task_1()); 
}

#[test]
fn test_db_fetch_current_one_break() {
    let mut selector = Top::new();

    let mut db = open_test_db();
    let tx = db.transaction().expect("Failed to begin transaction");
    
    tx.add_task(&example_task_break_1()).expect("Failed adding task to db in test");

    tx.select_current_task(&mut selector).expect("Failed to select current task");

    let res = tx.fetch_current_task();
    assert!(res.is_ok(), "Error fetching the current task: {}", res.unwrap_err());

    let opt = res.unwrap();
    assert!(opt.is_some(), "No current task after selecting one");

    let task = opt.unwrap();
    assert_eq!(task, example_task_break_1()); 
}

#[test]
fn test_db_fetch_current_task_vs_break() {
    let mut top_selector = Top::new();
    let mut random_break_selector = WeightedRandom::new(1.0);
    let mut random_task_selector = WeightedRandom::new(0.0);

    let mut db = open_test_db();
    let tx = db.transaction().expect("Failed to begin transaction");
    
    tx.add_task(&example_task_1()).expect("Failed adding task to db in test");
    tx.add_task(&example_task_2()).expect("Failed adding task to db in test");
    tx.add_task(&example_task_break_1()).expect("Failed adding task to db in test");

    // blocks are just for separating the tests, they are not required.

    { // test with top, choose example_task_2 because it has higher priority than 1
        tx.select_current_task(&mut top_selector).expect("Failed to select current task");
        let res = tx.fetch_current_task();
        assert!(res.is_ok(), "Error fetching the current task: {}", res.unwrap_err());

        let opt = res.unwrap();
        assert!(opt.is_some(), "No current task after selecting one");

        let task = opt.unwrap();
        assert_eq!(task, example_task_2()); 
    }

    { // test with weighted, breakp = 0
        tx.select_current_task(&mut random_task_selector).expect("Failed to select current task");
        let res = tx.fetch_current_task();
        assert!(res.is_ok(), "Error fetching the current task: {}", res.unwrap_err());

        let opt = res.unwrap();
        assert!(opt.is_some(), "No current task after selecting one");

        let task = opt.unwrap();
        assert!( (task == example_task_1()) || (task == example_task_2()) ); 
    }

    { // test with weighted, breakp = 1
        tx.select_current_task(&mut random_break_selector).expect("Failed to select current task");
        let res = tx.fetch_current_task();
        assert!(res.is_ok(), "Error fetching the current task: {}", res.unwrap_err());

        let opt = res.unwrap();
        assert!(opt.is_some(), "No current task after selecting one");

        let task = opt.unwrap();
        assert_eq!(task, example_task_break_1()); 
    }
}

#[test]
fn test_db_fetch_current_ordering_two() {
    let mut selector = Top::new();

    let mut db = open_test_db();
    let tx = db.transaction().expect("Failed to begin transaction");

    // two tasks with equal priority but names are different, second is exactly the same but ends
    // with a "2"
    let task1 = Task::new_from_parts("a".to_string(), 1, false).unwrap();
    let task2 = Task::new_from_parts("a2".to_string(), 1, false).unwrap();

    tx.add_task(&task1).expect("Failed adding task to db");
    tx.add_task(&task2).expect("Failed adding task to db");

    tx.select_current_task(&mut selector).expect("Failed to select current task");
    let res = tx.fetch_current_task();
    assert!(res.is_ok(), "Error fetching the current task: {}", res.unwrap_err());

    let opt = res.unwrap();
    assert!(opt.is_some(), "No current task after selecting one");

    let current = opt.unwrap();
    // FIXME: behavior is currently unspecified due to bug in sql statements - they don't have an
    // order by clause for the text of the tasks
    assert_eq!(current, task2); 
}

#[test]
fn test_db_fetch_current_two_max_u32() {
    let mut selector = Top::new();

    let mut db = open_test_db();
    let tx = db.transaction().expect("Failed to begin transaction");

    let task1 = Task::new_from_parts("a".to_string(), u32::max_value(), false).unwrap();
    let task2 = Task::new_from_parts("b".to_string(), u32::max_value(), false).unwrap();

    // add to db
    tx.add_task(&task1).expect("Failed adding task to db in test");
    tx.add_task(&task2).expect("Failed adding task to db in test");

    tx.select_current_task(&mut selector).expect("Failed to select current task");
    let res = tx.fetch_current_task();
    assert!(res.is_ok(), "Error fetching the current task: {}", res.unwrap_err());

    let opt = res.unwrap();
    assert!(opt.is_some(), "No current task after selecting one");

    let current = opt.unwrap();
    // FIXME: behavior is currently unspecified due to bug in sql statements - they don't have an
    // order by clause for the text of the tasks
    assert_eq!(current, task2); 
}


// FIXME: this test is fragile because the priority is the only thing we're ordering by, and we're
// relying on tasks returned in FIFO order from the db when they have the same priority
proptest! {
    #[test]
    fn test_db_fetch_current_arb(tasks in arb_task_list()) {
        let mut selector = Top::new();

        let mut db = open_test_db();
        let tx = db.transaction().expect("Failed to begin transaction");

        let (mut breaks, mut tasks): (Vec<_>, Vec<_>) = tasks.into_iter().partition(|t| t.is_break());
        breaks.sort_by_key(|t| t.priority());
        tasks.sort_by_key(|t| t.priority());

        // first add breaks, then add tasks, so that we can check that tasks will always be chosen
        // over breaks

        if !breaks.is_empty() {
            let mut max_break = &breaks[0];

            for task in &breaks {
                if task.priority() >= max_break.priority() {
                    max_break = task;
                }

                tx.add_task(&task).expect("Failed adding task to db");

                tx.select_current_task(&mut selector).expect("Failed to select current task");
                let res = tx.fetch_current_task();
                prop_assert!(res.is_ok(), "Error fetching the current task: {}", res.unwrap_err());

                let opt = res.unwrap();
                prop_assert!(opt.is_some(), "No current task after selecting one");

                let current = opt.unwrap();
                prop_assert_eq!(&current, max_break);
            }
        }

        if !tasks.is_empty() {
            let mut max_task = &tasks[0];

            for task in &tasks {
                if task.priority() >= max_task.priority() {
                    max_task = task;
                }

                tx.add_task(&task).expect("Failed adding task to db");

                tx.select_current_task(&mut selector).expect("Failed to select current task");
                let res = tx.fetch_current_task();
                prop_assert!(res.is_ok(), "Error fetching the current task: {}", res.unwrap_err());

                let opt = res.unwrap();
                prop_assert!(opt.is_some(), "No current task after selecting one");

                let current = opt.unwrap();
                prop_assert_eq!(&current, max_task);
            }
        }

    }
}
