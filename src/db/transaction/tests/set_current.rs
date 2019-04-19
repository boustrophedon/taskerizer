use crate::db::{DBBackend, DBTransaction};
use crate::db::tests::open_test_db;

use crate::task::test_utils::{example_task_1, arb_task_list};

#[test]
fn test_tx_set_current() {
    let task = example_task_1();

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    tx.add_task(&task).unwrap();
    let tasks = tx.fetch_tasks().unwrap();

    let res = tx.set_current_task(&tasks[0].0);
    assert!(res.is_ok(), "Error setting current task: {}", res.unwrap_err());
    tx.commit().unwrap();
}

proptest! {
    #[test]
    fn test_tx_set_current_arb(tasks in arb_task_list()) {
        let mut db = open_test_db();
        let tx = db.transaction().unwrap();

        // add all tasks
        for task in tasks {
            tx.add_task(&task).unwrap();
        }

        let tasks = tx.fetch_tasks().unwrap();
        let breaks = tx.fetch_breaks().unwrap();
        let all_task_ids = tasks.into_iter().map(|t| t.0).chain(breaks.into_iter().map(|t| t.0));

        // set current task to each task and check for error
        for id in all_task_ids {
            let res = tx.set_current_task(&id);
            prop_assert!(res.is_ok(), "Error setting current task: {}", res.unwrap_err());
        }
        tx.commit().unwrap();
    }
}
