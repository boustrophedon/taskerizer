use proptest::prelude::*;

use tempfile::{tempdir, TempDir};

use chrono::Utc;

use task::Task;
use db::{SqliteBackend, DBBackend};

// utility functions

// we have to return the TempDir so that it doesn't get dropped and therefore delete the db
fn open_test_db() -> (SqliteBackend, TempDir) {
    let test_dir = tempdir().expect("temporary directory could not be created");
 
    let res = SqliteBackend::open(&test_dir);
    assert!(res.is_ok(), "Error opening db: {}", res.unwrap_err());
    let db = res.unwrap();

    (db, test_dir)
}

// proptest gen functions

prop_compose! {
    fn arb_task()(task in any::<String>(),
                  priority in any::<u32>(),
                  reward in any::<bool>()) -> Task {
        Task {
            task: task,
            priority: priority,
            reward: reward,
        }
    }
}

prop_compose! {
    fn arb_task_list()(tasks in prop::collection::vec(arb_task(), 1..100))
        -> Vec<Task> {
            tasks
    }
}

// tests

#[test]
fn test_db_open() {
    open_test_db();
}

#[test]
fn test_db_metadata() {
    let before_creation = Utc::now();
    let (db, dir) = open_test_db();

    // close connection and record time
    db.close().expect("closing db connection failed");
    let after_creation = Utc::now();

    // open again and read metadata
    let db = SqliteBackend::open(&dir).expect("opening database failed");
    let metadata = db.metadata().expect("getting db metadata failed");

    let ver = env!("CARGO_PKG_VERSION");

    assert!(metadata.version == ver, "Versions do not match: db version {}, crate version {}", metadata.version, ver);
    assert!(metadata.date_created >= before_creation, "Database was created in the past");
    assert!(metadata.date_created < after_creation, "Database was created in the future");
}

#[test]
fn test_db_add() {
    let (db, dir) = open_test_db();

    let task = Task {
        task: "test task please ignore".to_string(),
        priority: 1,
        reward: false,
    };

    let reward = Task {
        task: "test task please ignore".to_string(),
        priority: 1,
        reward: true,
    };

    let res = db.add_task(&task);
    assert!(res.is_ok(), "Adding task failed: {:?}, err: {}", task, res.unwrap_err());
    let res = db.add_task(&reward);
    assert!(res.is_ok(), "Adding reward failed: {:?}, err: {}", reward, res.unwrap_err());
}

proptest! {
    #[test]
    fn test_db_add_arb(task1 in arb_task(),
                   task2 in arb_task()) {
        let (db, dir) = open_test_db();

        prop_assert!(db.add_task(&task1).is_ok(), "Adding task failed. task1: {:?}", task1);
        prop_assert!(db.add_task(&task2).is_ok(), "Adding task failed. task2: {:?}", task2);
    }
}

#[test]
fn test_db_list() {
    let (db, dir) = open_test_db();

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
        let (db, dir) = open_test_db();

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
