use crate::db::{DBBackend, DBTransaction};
use crate::db::tests::open_test_db;

use crate::task::test_utils::{example_task_1, example_task_break_1, arb_task_list};

#[test]
fn test_tx_fetch_current_task_empty() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let res = DBTransaction::fetch_current_task(&tx);
    assert!(res.is_ok(), "Error getting current task: {}", res.unwrap_err());

    let task_opt = res.unwrap();
    assert!(task_opt.is_none(), "Task inside database without inserting anything into it: {:?}", task_opt.unwrap());
}

#[test]
fn test_tx_fetch_current_task_1() {
    let task = example_task_1();
    let reward = example_task_break_1();

    // open db, insert one task and one break
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();
    tx.add_task(&task).unwrap();
    tx.add_task(&reward).unwrap();

    // get the task we inserted and set it as the current task
    let id = &tx.fetch_tasks().unwrap()[0].0;
    tx.set_current_task(id).unwrap();

    // get the current task and verify it's the one we set
    let res = DBTransaction::fetch_current_task(&tx); 
    assert!(res.is_ok(), "Error getting current task: {}", res.unwrap_err());

    let task_opt = res.unwrap();
    assert!(task_opt.is_some(), "No current task even though we set it.");

    let db_task = task_opt.unwrap();
    assert_eq!(db_task, task);
}

#[test]
fn test_tx_fetch_current_task_2() {
    let task = example_task_1();
    let reward = example_task_break_1();

    // open db, insert one task and one break
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();
    tx.add_task(&task).unwrap();
    tx.add_task(&reward).unwrap();

    // get the break we inserted and set it as the current task
    let id = &tx.fetch_breaks().unwrap()[0].0;
    tx.set_current_task(id).unwrap();

    // get the current task and verify it's the one we set
    let res = DBTransaction::fetch_current_task(&tx); 
    assert!(res.is_ok(), "Error getting current task: {}", res.unwrap_err());

    let task_opt = res.unwrap();
    assert!(task_opt.is_some(), "No current task even though we set it.");

    let db_task = task_opt.unwrap();
    assert_eq!(db_task, reward);
}

proptest! {
    #[test]
    fn test_tx_fetch_current_task_arb(tasks in arb_task_list()) {
        // open db, insert all tasks
        let mut db = open_test_db();
        let tx = db.transaction().unwrap();

        for task in tasks {
            tx.add_task(&task).unwrap();
        }
        
        // get tasks and breaks
        let db_tasks = tx.fetch_tasks().unwrap();
        let db_breaks = tx.fetch_breaks().unwrap();
        let all_db_tasks = db_tasks.into_iter().chain(db_breaks);

        // set each task and break to current and check we get back the right one
        for (id, db_task) in all_db_tasks {
            tx.set_current_task(&id).unwrap();

            // verify we get back the one we set
            let res = DBTransaction::fetch_current_task(&tx); 
            prop_assert!(res.is_ok(), "Error getting current task: {}", res.unwrap_err());

            let task_opt = res.unwrap();
            prop_assert!(task_opt.is_some(), "No current task even though we set it.");

            let current_task = task_opt.unwrap();
            prop_assert_eq!(current_task, db_task);
        }
    }
}
