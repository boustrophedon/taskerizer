use crate::db::DBTransaction;
use crate::db::tests::open_test_db;

use crate::task::test_utils::{example_task_1, example_task_break_1, arb_task_list};

// these tests were mostly copied from the get_current_task tests

#[test]
fn test_tx_pop_current_task_empty() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let res = tx.pop_current_task();
    assert!(res.is_ok(), "Error popting current task: {}", res.unwrap_err());

    let task_opt = res.unwrap();
    assert!(task_opt.is_none(), "There is a task set as current without inserting anything into it: {:?}", task_opt.unwrap());
}

#[test]
fn test_tx_pop_current_task_1() {
    let task = example_task_1();
    let reward = example_task_break_1();

    // open db, insert one task and one break
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();
    tx.add_task(&task).unwrap();
    tx.add_task(&reward).unwrap();

    // get the task we inserted and set it as the current task
    let id = tx.get_tasks().unwrap()[0].0;
    tx.set_current_task(&id).unwrap();

    // pop the current task and verify it's the one we set
    let res = tx.pop_current_task(); 
    assert!(res.is_ok(), "Error getting current task: {}", res.unwrap_err());

    let id_opt = res.unwrap();
    assert!(id_opt.is_some(), "No current task was popped even though we set it.");

    let db_task_id = id_opt.unwrap();
    assert_eq!(db_task_id, id);
}

#[test]
fn test_tx_pop_current_task_2() {
    let task = example_task_1();
    let reward = example_task_break_1();

    // open db, insert one task and one break
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();
    tx.add_task(&task).unwrap();
    tx.add_task(&reward).unwrap();

    // get the break we inserted and set it as the current task
    let id = tx.get_breaks().unwrap()[0].0;
    tx.set_current_task(&id).unwrap();

    // pop the current task and verify it's the one we set
    let res = tx.pop_current_task(); 
    assert!(res.is_ok(), "Error getting current task: {}", res.unwrap_err());

    let id_opt = res.unwrap();
    assert!(id_opt.is_some(), "No current task was popped even though we set it.");

    let db_task_id = id_opt.unwrap();
    assert_eq!(db_task_id, id);
}

proptest! {
    #[test]
    fn test_tx_pop_current_task_arb(tasks in arb_task_list()) {
        // open db, insert all tasks
        let mut db = open_test_db();
        let tx = db.transaction().unwrap();

        for task in tasks {
            tx.add_task(&task).unwrap();
        }
        
        // get tasks and breaks
        let db_tasks = tx.get_tasks().unwrap();
        let db_breaks = tx.get_breaks().unwrap();
        let all_db_tasks = db_tasks.into_iter().chain(db_breaks);

        // set each task and break to current and check we get back the right one
        for (id, _) in all_db_tasks {
            tx.set_current_task(&id).unwrap();

            // verify we get back the one we set
            let res = tx.pop_current_task(); 
            prop_assert!(res.is_ok(), "Error getting current task: {}", res.unwrap_err());

            let id_opt = res.unwrap();
            prop_assert!(id_opt.is_some(), "No current task was popped even though we set it.");

            let db_task_id = id_opt.unwrap();
            prop_assert_eq!(db_task_id, id);
        }
    }
}
