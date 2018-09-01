use task::Task;
use db::DBBackend;

use super::open_test_db;
use super::arb_task_list;

#[test]
fn test_db_list_empty() {
    let (db, _dir) = open_test_db();

    // get nothing from db
    let res = db.get_all_tasks();
    assert!(res.is_ok(), "Tasks could not be retrieved: {:?}", res.unwrap_err());
    let db_tasks = res.unwrap();

    // check nothing was returned
    assert_eq!(db_tasks.len(), 0);
}

#[test]
fn test_db_list_added_manually() {
    let (db, _dir) = open_test_db();

    // manually make a list of tasks

    let mut tasks = Vec::new();
    tasks.push( Task {
        task: "test task please ignore".to_string(),
        priority: 11,
        reward: false,
    });

    tasks.push( Task {
        task: "test task 2".to_string(),
        priority: 12,
        reward: true,
    });

    tasks.push( Task {
        task: "test task 3".to_string(),
        priority: 13,
        reward: false,
    });

    tasks.push( Task {
        task: "test task 4".to_string(),
        priority: 14,
        reward: true,
    });

    // add all tasks to db
    for task in &tasks {
        let res = db.add_task(&task);
        assert!(res.is_ok(), "Adding task failed. task: {:?}, err: {}", task, res.unwrap_err());
    }

    // get tasks back from db
    let res = db.get_all_tasks();
    assert!(res.is_ok(), "Tasks could not be retrieved: {:?}", res.unwrap_err());
    let db_tasks = res.unwrap();

    // check number of tasks returned is correct
    assert_eq!(db_tasks.len(), tasks.len());

    // check every task made it back
    for task in &tasks {
        assert!(db_tasks.contains(task), "tasks returned from db does not contain task {:?}", task);
    }

}

proptest! {
    #[test]
    fn test_db_list_arb(tasks in arb_task_list()) {
        let (db, _dir) = open_test_db();

        // add all tasks to db
        for task in &tasks {
            let res = db.add_task(&task);
            assert!(res.is_ok(), "Adding task failed. task: {:?}, err: {}", task, res.unwrap_err());
        }

        // get tasks back
        let res = db.get_all_tasks();
        assert!(res.is_ok(), "Tasks could not be retrieved: {:?}", res.unwrap_err());
        let db_tasks = res.unwrap();

        // check number of tasks returned is correct
        assert_eq!(db_tasks.len(), tasks.len());

        // check every task made it back
        for task in &tasks {
            assert!(db_tasks.contains(task), "tasks returned from db does not contain task {:?}", task);
        }
    }
}
