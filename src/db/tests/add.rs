use db::DBBackend;

use super::open_test_db;

use task::test_utils::{example_task_1, example_task_break_1, arb_task};

#[test]
fn test_db_add() {
    let mut db = open_test_db();

    let task = example_task_1();
    let reward = example_task_break_1();

    let res = db.add_task(&task);
    assert!(res.is_ok(), "Adding task failed: {:?}, err: {}", task, res.unwrap_err());
    let res = db.add_task(&reward);
    assert!(res.is_ok(), "Adding task with break failed: {:?}, err: {}", reward, res.unwrap_err());
}

proptest! {
    #[test]
    fn test_db_add_arb(task1 in arb_task(),
                   task2 in arb_task()) {
        let mut db = open_test_db();

        prop_assert!(db.add_task(&task1).is_ok(), "Adding task failed. task1: {:?}", task1);
        prop_assert!(db.add_task(&task2).is_ok(), "Adding task failed. task2: {:?}", task2);
    }
}
