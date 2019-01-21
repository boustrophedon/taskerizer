use crate::commands::test_utils::add_from_task;

use crate::db::DBBackend;
use crate::db::tests::open_test_db;

use crate::task::Task;
use crate::task::test_utils::{example_task_1, example_task_list, arb_task, arb_task_list};

use crate::selection::WeightedRandom;

/// Check that two tasks are equal, ignoring the UUID. This is useful here because we are
/// pretending to make user input, so we don't know the UUID in advance.
fn eq_ignore_uuid(task1: &Task, task2: &Task) -> bool {
    task1.task() == task2.task() &&
    task1.priority() == task2.priority() &&
    task1.is_break() == task2.is_break()
}

#[test]
/// Add a task, check that the current task is the right one in the tx.
fn test_runcmd_add_current() {
    // break probability = 0
    let mut selector = WeightedRandom::new(0.0);

    let mut db = open_test_db();
    let tx = db.transaction().expect("Failed to begin transaction");

    let task = example_task_1();
    let cmd = add_from_task(&task);

    cmd.run(&tx, &mut selector).expect("Add command failed");

    let current = tx.fetch_current_task().expect("Failed getting current task from db");
    assert!(current.is_some(), "No current task after running Add command");

    let current = current.unwrap();
    assert!(eq_ignore_uuid(&current, &task));
}

#[test]
/// Add a task, check that the current task is the right one in the db, then add more and check
/// that the current is still the first task.
fn test_runcmd_add_multiple_current() {
    // break probability = 0
    let mut selector = WeightedRandom::new(0.0);

    let mut db = open_test_db();
    let tx = db.transaction().expect("Failed to begin transaction");

    let task = example_task_1();
    let cmd = add_from_task(&task);

    cmd.run(&tx, &mut selector).expect("Add command failed");

    let current = tx.fetch_current_task().expect("Failed getting current task from db");
    assert!(current.is_some(), "No current task after running Add command");

    let current = current.unwrap();
    assert!(eq_ignore_uuid(&current, &task));

    // add more tasks, checking that the current task doesn't change
    for other_task in example_task_list() {
        let cmd = add_from_task(&other_task);
        cmd.run(&tx, &mut selector).expect("Add command failed");

        let current = tx.fetch_current_task().expect("Failed getting current task from db");
        assert!(current.is_some(), "No current task after running Add command");

        let current = current.unwrap();
        assert!(eq_ignore_uuid(&current, &task)); // note that this is the first task, from example_task_1()
    }

}


proptest! {
    #[test]
    /// Proptest with same code as above
    fn test_runcmd_add_current_arb(task in arb_task()) {
        // break probability = 0
        let mut selector = WeightedRandom::new(0.0);

        let mut db = open_test_db();
        let tx = db.transaction().expect("Failed to begin transaction");

        let cmd = add_from_task(&task);

        cmd.run(&tx, &mut selector).expect("Add command failed");

        let current = tx.fetch_current_task().expect("Failed getting current task from db");
        prop_assert!(current.is_some(), "No current task after running Add command");

        let current = current.unwrap();
        prop_assert!(eq_ignore_uuid(&current, &task));
    }
}

proptest! {
    #[test]
    /// Proptest with same code as above
    fn test_runcmd_add_multiple_current_arb(task in arb_task(), tasks in arb_task_list()) {
        // break probability = 0
        let mut selector = WeightedRandom::new(0.0);

        let mut db = open_test_db();
        let tx = db.transaction().expect("Failed to begin transaction");

        let cmd = add_from_task(&task);

        cmd.run(&tx, &mut selector).expect("Add command failed");

        let current = tx.fetch_current_task().expect("Failed getting current task from db");
        prop_assert!(current.is_some(), "No current task after running Add command");

        let current = current.unwrap();
        prop_assert!(eq_ignore_uuid(&current, &task));

        // add more tasks, checking that the current task doesn't change
        for other_task in tasks {
            let cmd = add_from_task(&other_task);
            cmd.run(&tx, &mut selector).expect("Add command failed");

            let current = tx.fetch_current_task().expect("Failed getting current task from db");
            prop_assert!(current.is_some(), "No current task after running Add command");

            let current = current.unwrap();
            prop_assert!(eq_ignore_uuid(&current, &task)); // note that this is the first task, from arb_task()
        }
    }
}
