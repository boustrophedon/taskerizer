use db::DBBackend;

use super::open_test_db;

use task::test_utils::{example_task_1, example_task_break_1, arb_task_list};

use failure::Error;

// TODO besides the error tests, I'm not sure how useful these tests are. see ideas.txt

/// When we try to choose the current task from a category with no tasks in it, make sure we get
/// the correct error.
fn assert_no_task_found_error(res: Result<(), Error>) {
    assert!(res.is_err(), "Choosing current task with no existing tasks succeeded.");

    let err = res.unwrap_err();
    assert!(err.to_string().contains("No tasks with given category were found"), 
            "Not the expected error when choosing task with no tasks available: {}", err);
}

#[test]
fn test_db_choose_current_error_p() {
    let mut db = open_test_db();

    // test with p less than 0
    let res = db.choose_current_task(-0.1, false);
    assert!(res.is_err(), "Passed parameter less than 0 but did not error, got {:?}", res.unwrap());

    let err = res.unwrap_err();
    assert!(err.to_string().contains("p parameter was less than 0"));

    // test with p greater than 1
    let res = db.choose_current_task(1.1, false);
    assert!(res.is_err(), "Passed parameter greater than 1 but did not error, got {:?}", res.unwrap());

    let err = res.unwrap_err();
    assert!(err.to_string().contains("p parameter was greater than 1"));
}

#[test]
fn test_db_choose_current_empty() {
    let mut db = open_test_db();

    let res = db.choose_current_task(0.0, false);
    assert_no_task_found_error(res);
}

#[test]
fn test_db_choose_current_one_task() {
    let mut db = open_test_db();

    db.add_task(&example_task_1()).expect("Adding task failed");

    let res = db.choose_current_task(0.0, false);
    assert!(res.is_ok(), "Choosing current task with one task failed: {}", res.unwrap_err());

    let res = db.choose_current_task(0.0, true);
    assert_no_task_found_error(res);
}

#[test]
fn test_db_choose_current_one_break() {
    let mut db = open_test_db();

    db.add_task(&example_task_break_1()).expect("Adding task failed");

    let res = db.choose_current_task(0.0, true);
    assert!(res.is_ok(), "Choosing current break task with one break failed: {}", res.unwrap_err());

    let res = db.choose_current_task(0.0, false);
    assert_no_task_found_error(res);
}

proptest! {
    #[test]
    fn test_db_choose_current_arb(tasks in arb_task_list()) {
        let mut db = open_test_db();

        // keep track of which categories of tasks we have
        let mut has_break = false;
        let mut has_task = false;

        for task in &tasks {
            if task.is_break() {
                has_break = true;
            } else {
                has_task = true;
            }
            db.add_task(task).expect("adding task failed");
        }

        let res_first = db.choose_current_task(0.0, true);
        let res_last = db.choose_current_task(1.0, true);
        if has_break {
            assert!(res_first.is_ok(), "Choosing first current break failed: {}", res_first.unwrap_err());
            assert!(res_last.is_ok(), "Choosing last current break failed: {}", res_last.unwrap_err());
        }
        else {
            assert_no_task_found_error(res_first);
            assert_no_task_found_error(res_last);
        }


        let res_first = db.choose_current_task(0.0, false);
        let res_last = db.choose_current_task(1.0, false);
        if has_task {
            assert!(res_first.is_ok(), "Choosing first current task failed: {}", res_first.unwrap_err());
            assert!(res_last.is_ok(), "Choosing last current task failed: {}", res_last.unwrap_err());
        }
        else {
            assert_no_task_found_error(res_first);
            assert_no_task_found_error(res_last);
        }

    }
}
