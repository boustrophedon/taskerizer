use tempfile::{tempdir, TempDir};

use chrono::Utc;

use task::Task;
use db::{SqliteBackend, DBBackend};

fn open_test_db() -> (SqliteBackend, TempDir) {
    let test_dir = tempdir().expect("temporary directory could not be created");

    // expect is easier than getting the err out of the result and asserting it
    let db = SqliteBackend::open(&test_dir).expect("creating database failed");

    (db, test_dir)
}


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

    db.add(task).expect("adding task failed");
    db.add(reward).expect("adding reward failed");
}
