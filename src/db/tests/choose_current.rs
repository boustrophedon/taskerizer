use db::DBBackend;

use super::open_test_db;

use task::test_utils::{example_task_1, example_task_break_1, arb_task_list};

// TODO besides the error tests, I'm not sure how useful these tests are. see ideas.txt

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
    assert!(res.is_ok(), "Choosing current task with no existing tasks failed: {}", res.unwrap_err());
}

#[test]
fn test_db_choose_current_one_task() {
    let mut db = open_test_db();

    db.add_task(&example_task_1()).expect("Adding task failed");

    let res = db.choose_current_task(0.0, false);
    assert!(res.is_ok(), "Choosing current task with one task failed: {}", res.unwrap_err());

    let res = db.choose_current_task(0.0, true);
    assert!(res.is_ok(), "Choosing current reward task with one task failed: {}", res.unwrap_err());
}

#[test]
fn test_db_choose_current_one_break() {
    let mut db = open_test_db();

    db.add_task(&example_task_break_1()).expect("Adding task failed");

    let res = db.choose_current_task(0.0, false);
    assert!(res.is_ok(), "Choosing current task with one break failed: {}", res.unwrap_err());

    let res = db.choose_current_task(0.0, true);
    assert!(res.is_ok(), "Choosing current break task with one break failed: {}", res.unwrap_err());
}

proptest! {
    #[test]
    fn test_db_choose_current_arb(tasks in arb_task_list()) {
        let mut db = open_test_db();

        for task in &tasks {
            db.add_task(task).expect("adding task failed");
        }

        let res = db.choose_current_task(0.0, true);
        assert!(res.is_ok(), "Choosing first current break failed: {}", res.unwrap_err());
        let res = db.choose_current_task(0.0, false);
        assert!(res.is_ok(), "Choosing first current task failed: {}", res.unwrap_err());

        let res = db.choose_current_task(1.0, true);
        assert!(res.is_ok(), "Choosing last current break failed: {}", res.unwrap_err());
        let res = db.choose_current_task(1.0, false);
        assert!(res.is_ok(), "Choosing last current task failed: {}", res.unwrap_err());

    }
}
