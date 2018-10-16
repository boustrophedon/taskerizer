use commands::test_utils::add_from_task;
use db::DBBackend;
use db::tests::open_test_db;
use task::test_utils::{example_task_1, example_task_list, arb_task, arb_task_list};

// probabilities used to choose task vs break when selecting new current task
// first 0.0 implies we choose the first item in the list of tasks
// in order to choose a break, the second number (category_p) must be less than the third number
// (break_cutoff), and vice versa to choose a task.
// note that for these tests, it doesn't matter which we use because 1) if there is no task in a
// given category we just choose the other, and 2) we never change the current task in these tests
//const FIRST_BREAK = (0.0, 0.0, 1.0);
const FIRST_TASK: (f32, f32, f32) = (0.0, 1.0, 0.0);


#[test]
/// Add a task, check that the current task is the right one in the db.
fn test_runcmd_add_current() {
    let mut db = open_test_db();

    let task = example_task_1();
    let cmd = add_from_task(&task);

    cmd.run(&mut db, FIRST_TASK).expect("Add command failed");

    let current = db.get_current_task().expect("Failed getting current task from db");
    assert!(current.is_some(), "No current task after running Add command");

    let current = current.unwrap();
    assert_eq!(current, task);
}

#[test]
/// Add a task, check that the current task is the right one in the db, then add more and check
/// that the current is still the first task.
fn test_runcmd_add_multiple_current() {
    let mut db = open_test_db();

    let task = example_task_1();
    let cmd = add_from_task(&task);

    cmd.run(&mut db, FIRST_TASK).expect("Add command failed");

    let current = db.get_current_task().expect("Failed getting current task from db");
    assert!(current.is_some(), "No current task after running Add command");

    let current = current.unwrap();
    assert_eq!(current, task);

    // add more tasks, checking that the current task doesn't change
    for other_task in example_task_list() {
        let cmd = add_from_task(&other_task);
        cmd.run(&mut db, FIRST_TASK).expect("Add command failed");

        let current = db.get_current_task().expect("Failed getting current task from db");
        assert!(current.is_some(), "No current task after running Add command");

        let current = current.unwrap();
        assert_eq!(current, task); // note that this is the first task, from example_task_1()
    }

}


proptest! {
    #[test]
    /// Proptest with same code as above
    fn test_runcmd_add_current_arb(task in arb_task()) {
        let mut db = open_test_db();

        let cmd = add_from_task(&task);

        cmd.run(&mut db, FIRST_TASK).expect("Add command failed");

        let current = db.get_current_task().expect("Failed getting current task from db");
        prop_assert!(current.is_some(), "No current task after running Add command");

        let current = current.unwrap();
        prop_assert_eq!(current, task);
    }
}

proptest! {
    #[test]
    /// Proptest with same code as above
    fn test_runcmd_add_multiple_current_arb(task in arb_task(), tasks in arb_task_list()) {
        let mut db = open_test_db();

        let cmd = add_from_task(&task);

        cmd.run(&mut db, FIRST_TASK).expect("Add command failed");

        let current = db.get_current_task().expect("Failed getting current task from db");
        prop_assert!(current.is_some(), "No current task after running Add command");

        let current = current.unwrap();
        prop_assert_eq!(&current, &task);

        // add more tasks, checking that the current task doesn't change
        for other_task in tasks {
            let cmd = add_from_task(&other_task);
            cmd.run(&mut db, FIRST_TASK).expect("Add command failed");

            let current = db.get_current_task().expect("Failed getting current task from db");
            assert!(current.is_some(), "No current task after running Add command");

            let current = current.unwrap();
            prop_assert_eq!(&current, &task); // note that this is the first task, from arb_task()
        }
    }
}
