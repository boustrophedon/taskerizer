use crate::db::{DBBackend, SqliteBackend};

use crate::db::tests::open_test_db;

use crate::task::Task;
use crate::task::test_utils::{example_task_1, example_task_break_1, arb_task_list};

use proptest::test_runner::TestCaseError;

// utility for checking correct current task is selected
fn assert_task_at_p(db: &mut SqliteBackend, p: f32, expected_task: &Task, msg: &str) {
    db.choose_current_task(p, expected_task.is_break()).expect("Failed choosing current task");

    let res = db.get_current_task();
    assert!(res.is_ok(), "Getting current task failed: {}", res.unwrap_err());

    let task_opt = res.unwrap();
    assert!(task_opt.is_some(), "No current task even though we selected one.");

    let task = task_opt.unwrap();
    assert_eq!(task, *expected_task, "{}", msg);
}

// utility for checking correct current task is selected
fn prop_assert_task_at_p(db: &mut SqliteBackend, p: f32, expected_task: &Task, msg: &str) -> Result<(), TestCaseError> {
    db.choose_current_task(p, expected_task.is_break()).expect("Failed choosing current task");

    let res = db.get_current_task();
    prop_assert!(res.is_ok(), "Getting current task failed: {}", res.unwrap_err());

    let task_opt = res.unwrap();
    prop_assert!(task_opt.is_some(), "No current task even though we selected one.");

    let task = task_opt.unwrap();
    prop_assert_eq!(&task, expected_task, "{}", msg);

    Ok(())
}


#[test]
fn test_db_get_current_no_tasks() {
    let mut db = open_test_db();

    let res = db.get_current_task();
    assert!(res.is_ok(), "Getting current task failed: {}", res.unwrap_err());

    let no_task = res.unwrap();
    assert!(no_task.is_none(), "Without adding a task, somehow there was a current task: {:?}", no_task);
}

#[test]
fn test_db_get_current_one_task() {
    let mut db = open_test_db();
    
    db.add_task(&example_task_1()).expect("Failed adding task to db in test");

    assert_task_at_p(&mut db, 0.0, &example_task_1(), "example task 1");

    // also assert that we error if we try to choose current task from breaks if there are none

    let res = db.choose_current_task(0.0, true);
    assert!(res.is_err(), "DB did not return an error when choosing a new break without any breaks in db.");
    let err = res.unwrap_err();
    assert!(err.to_string().contains("No tasks with given category were found"), 
            "Not the expected error when choosing task with no tasks in category: {}", err);
}

#[test]
fn test_db_get_current_one_break() {
    let mut db = open_test_db();
    
    db.add_task(&example_task_break_1()).expect("Failed adding task to db in test");

    assert_task_at_p(&mut db, 0.0, &example_task_break_1(), "example break 1");

    // also assert that we error if we try to choose current task from tasks if there are none

    let res = db.choose_current_task(0.0, false);
    assert!(res.is_err(), "DB did not return an error when choosing a new break without any breaks in db.");
    let err = res.unwrap_err();
    assert!(err.to_string().contains("No tasks with given category were found"),
            "Not the expected error when choosing task with no tasks in category: {}", err);
}

#[test]
fn test_db_get_current_task_vs_break() {
    let mut db = open_test_db();
    
    db.add_task(&example_task_1()).expect("Failed adding task to db in test");
    db.add_task(&example_task_break_1()).expect("Failed adding task to db in test");

    // select break
    assert_task_at_p(&mut db, 0.0, &example_task_break_1(), "example break 1");
    assert_task_at_p(&mut db, 0.3, &example_task_break_1(), "example break 1");
    assert_task_at_p(&mut db, 0.6, &example_task_break_1(), "example break 1");
    assert_task_at_p(&mut db, 1.0, &example_task_break_1(), "example break 1");

    // select task
    assert_task_at_p(&mut db, 0.0, &example_task_1(), "example task 1");
    assert_task_at_p(&mut db, 0.3, &example_task_1(), "example task 1");
    assert_task_at_p(&mut db, 0.6, &example_task_1(), "example task 1");
    assert_task_at_p(&mut db, 1.0, &example_task_1(), "example task 1");
}

#[test]
fn test_db_get_current_ordering_two() {
    let mut db = open_test_db();

    // two tasks with equal priority but names are different, second is exactly the same but ends
    // with a "2"
    let task1 = Task::from_parts("a".to_string(), 1, false).unwrap();
    let task2 = Task::from_parts("a2".to_string(), 1, false).unwrap();

    db.add_task(&task1).expect("Failed adding task to db");
    db.add_task(&task2).expect("Failed adding task to db");

    // task 1 is first because priority and text is same except for extra "2" at end of task2 text
    assert_task_at_p(&mut db, 0.0, &task1, "task 1");
    assert_task_at_p(&mut db, 0.49, &task1, "task 1");

    assert_task_at_p(&mut db, 0.51, &task2, "task 2");
    assert_task_at_p(&mut db, 1.0, &task2, "task 2");
}

#[test]
fn test_db_get_current_two_max_u32() {
    let mut db = open_test_db();

    let task1 = Task::from_parts("a".to_string(), u32::max_value(), false).unwrap();
    let task2 = Task::from_parts("b".to_string(), u32::max_value(), false).unwrap();

    // add to db
    db.add_task(&task1).expect("Failed adding task to db in test");
    db.add_task(&task2).expect("Failed adding task to db in test");

    // task1 is the first because they have the same priority and "a" < "b"
    assert_task_at_p(&mut db, 0.0, &task1, "task 1 max priority");
    assert_task_at_p(&mut db, 0.49, &task1, "task 1 max priority");

    assert_task_at_p(&mut db, 0.51, &task2, "task 2 max priority");
    assert_task_at_p(&mut db, 1.0, &task2, "task 2 max priority");
}


// TODO this test could be better probably. 
proptest! {
    #[test]
    fn test_db_get_current_arb(tasks in arb_task_list()) {
        let mut db = open_test_db();

        for task in &tasks {
            db.add_task(&task).expect("Failed adding task to db");
        }

        let (breaks, tasks): (Vec<Task>, Vec<Task>) = tasks.into_iter().partition(|task| task.is_break());

        use std::cmp::Ordering;
        // compare first by priority and then by text
        fn cmp_tasks(t1: &Task, t2: &Task) -> Ordering {
            t1.priority().cmp(&t2.priority()).then(t1.task().cmp(&t2.task()))
        }

        let min_task = tasks.iter().cloned().into_iter().min_by(cmp_tasks);
        let max_task = tasks.into_iter().max_by(cmp_tasks);
        let min_break = breaks.iter().cloned().into_iter().min_by(cmp_tasks);
        let max_break = breaks.into_iter().max_by(cmp_tasks);

        // assert task at p=0.0 has min priority
        if let Some(min_task) = min_task {
            prop_assert_task_at_p(&mut db, 0.0, &min_task, "min_task")?;
        }
        // assert task at p=1.0 has max priority
        if let Some(max_task) = max_task {
            prop_assert_task_at_p(&mut db, 1.0, &max_task, "max_task")?;
        }

        // assert break at p=0.0 has min priority
        if let Some(min_break) = min_break {
            prop_assert_task_at_p(&mut db, 0.0, &min_break, "min_break")?;
        }

        // assert break at p=1.0 has max priority
        if let Some(max_break) = max_break {
            prop_assert_task_at_p(&mut db, 1.0, &max_break, "max_break")?;
        }
    }
}
