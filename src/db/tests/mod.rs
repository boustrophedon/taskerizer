use proptest::prelude::*;

use tempfile::{tempdir, TempDir};

use chrono::Utc;

use task::Task;
use db::{SqliteBackend, DBBackend};

// utility functions

/// Open an in-memory db for testing
fn open_test_db() -> SqliteBackend {
    let res = SqliteBackend::open_in_memory();
    assert!(res.is_ok(), "Failed to open sqlite backend in-memory: {}", res.unwrap_err());
    res.unwrap()
}

/// Opens an on-disk database for testing
/// Returns the TempDir so that it does not go out of scope and delete the database file
fn open_test_db_on_disk() -> (SqliteBackend, TempDir) {
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
fn test_db_open_on_disk() {
    open_test_db_on_disk();
}

#[test]
fn test_db_open_err_bad_dir() {
    let test_dir = tempdir().expect("temporary directory could not be created");

    let mut bad_dir = test_dir.path().to_path_buf();
    bad_dir.push("bad");
    let res = SqliteBackend::open(&bad_dir);

    assert!(res.is_err(), "DB incorrectly opened without error: {:?}", res.unwrap());
}

#[test]
/// This test tests both that the metadata is written and that it is stored to disk. It acts as
/// both a metadata table check and a sanity "is stuff being written to disk" check.
fn test_db_metadata() {
    let before_creation = Utc::now();
    let (db, dir) = open_test_db_on_disk();

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

mod add;
mod list;

mod utils;
