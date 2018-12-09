//use db::{DBTransaction, SqliteTransaction};
//use db::tests::open_test_db;
//
//use task::test_utils::example_task_1;
//
//#[test]
//fn test_tx_add_task() {
//    let mut db = open_test_db();
//
//    let task = example_task_1();
//
//    let tx = db.transaction();
//
//    let res = tx.add_task(&task);
//    assert!(res.is_ok(), "Adding task failed: {}", res.unwrap_err());
//}
