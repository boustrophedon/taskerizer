use crate::db::DBTransaction;
use crate::db::tests::open_test_db;

use crate::task::test_utils::{example_task_1, arb_task_list};

// These tests don't really do anything. They just check that "if you add a valid task, the db does
// not error."
//
// TODO add an invalid task (eg null bytes in string) and check that it errors?
//
// Alternatively, pass through the number of rows changed from the sql calls and check that it's
// correct.

#[test]
fn test_tx_add_task_rollback() {
    let task = example_task_1();

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let res = tx.add_task(&task);
    assert!(res.is_ok(), "Adding task failed: {}", res.unwrap_err());

    let res = tx.rollback();
    assert!(res.is_ok(), "Rolling back transaction failed: {}", res.unwrap_err());
}

#[test]
fn test_tx_add_task_commit() {
    let task = example_task_1();

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let res = tx.add_task(&task);
    assert!(res.is_ok(), "Adding task failed: {}", res.unwrap_err());

    let res = tx.commit();
    assert!(res.is_ok(), "Committing transaction failed: {}", res.unwrap_err());
}

proptest! {
    #[test]
    fn test_tx_add_task_rollback_arb(tasks in arb_task_list()) {
        let mut db = open_test_db();
        let tx = db.transaction().unwrap();

        for task in tasks {
            let res = tx.add_task(&task);
            assert!(res.is_ok(), "Adding task failed: {}", res.unwrap_err());
        }
        let res = tx.rollback();
        assert!(res.is_ok(), "Rolling back transaction failed: {}", res.unwrap_err());
    }
}

proptest! {
    #[test]
    fn test_tx_add_task_commit_arb(tasks in arb_task_list()) {
        let mut db = open_test_db();
        let tx = db.transaction().unwrap();

        for task in tasks {
            let res = tx.add_task(&task);
            assert!(res.is_ok(), "Adding task failed: {}", res.unwrap_err());
        }
        let res = tx.commit();
        assert!(res.is_ok(), "Committing transaction failed: {}", res.unwrap_err());
    }
}
