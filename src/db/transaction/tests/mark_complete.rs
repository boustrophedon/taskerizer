use crate::db::DBTransaction;
use crate::db::tests::open_test_db;

use crate::task::test_utils::{example_task_1, arb_task_list};

// These tests don't really do anything. They just check that "if you mark a task complete, the db does
// not error."

#[test]
fn test_tx_mark_task_complete() {
    let task = example_task_1();

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    tx.add_task(&task).expect("Couldn't add task");
    let task_id = tx.fetch_tasks().expect("couldn't fetch tasks")[0].0;

    let res = tx.mark_task_complete(&task_id);
    assert!(res.is_ok(), "Marking task complete failed: {}", res.unwrap_err());

    let res = tx.commit();
    assert!(res.is_ok(), "Committing transaction failed: {}", res.unwrap_err());
}

#[test]
fn test_tx_mark_task_complete_duplicate_uuid_error() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let task = example_task_1();
    tx.add_task(&task).expect("Couldn't add task");
    let task_id = tx.fetch_tasks().expect("couldn't fetch tasks")[0].0;

    tx.mark_task_complete(&task_id).expect("Adding task failed");
    let res = tx.mark_task_complete(&task_id);

    // the second time it should fail
    assert!(res.is_err(), "Marking the same task complete twice (with the same UUID) did not result in an error.");
 
    let err = res.unwrap_err();
    assert!(err.to_string().contains("UNIQUE constraint failed: completed.uuid"),
            "Incorrect error message when inserting duplicate task: got {}", err);
}

proptest! {
    #[test]
    fn test_tx_mark_task_complete_arb(tasks in arb_task_list()) {
        let mut db = open_test_db();
        let tx = db.transaction().unwrap();

        for task in tasks {
            tx.add_task(&task).expect("Couldn't add task");
        }
        let tasks = tx.fetch_tasks().expect("couldn't fetch tasks");
        let breaks = tx.fetch_breaks().expect("Couldn't fetch breaks");
        for (task_id, _) in tasks.iter().chain(&breaks) {
            let res = tx.mark_task_complete(&task_id);
            prop_assert!(res.is_ok(), "Marking task complete failed: {}", res.unwrap_err());
        }
        let res = tx.commit();
        prop_assert!(res.is_ok(), "Committing transaction failed: {}", res.unwrap_err());
    }
}
