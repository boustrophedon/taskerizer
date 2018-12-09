use db::{DBTransaction, SqliteTransaction};
use db::tests::open_test_db;

use task::test_utils::example_task_1;

#[test]
fn test_db_tx_add_task_rollback() {
    let task = example_task_1();

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let res = tx.add_task(&task);
    assert!(res.is_ok(), "Adding task failed: {}", res.unwrap_err());

    let res = tx.rollback();
    assert!(res.is_ok(), "Rolling back transaction failed: {}", res.unwrap_err());
}

#[test]
fn test_db_tx_add_task_commit() {
    let task = example_task_1();

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let res = tx.add_task(&task);
    assert!(res.is_ok(), "Adding task failed: {}", res.unwrap_err());

    let res = tx.commit();
    assert!(res.is_ok(), "Committing transaction failed: {}", res.unwrap_err());
}
