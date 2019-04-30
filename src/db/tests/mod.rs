use std::env;

use tempfile::{tempdir, TempDir};

use chrono::Utc;

use crate::db::{SqliteBackend, DBBackend};

mod add;
mod select_current;
mod fetch_current;
mod list;
mod complete;
mod skip;
mod remove_by_uuid;

mod store_uset_op;
mod fetch_uset_op;
mod clear_uset_op;

mod store_replica;
mod store_replica_server;
mod fetch_replicas;

use proptest::prelude::*;
use crate::sync::ReplicaUuid;
use uuid::Uuid;

// for testing store/fetch replica server data

prop_compose! {
    fn arb_replica_ids()(size in 0..50usize)
        (uuids in prop::collection::vec(any::<u128>(), size))
        -> Vec<ReplicaUuid> {
            uuids.into_iter().map(|id| Uuid::from(id)).collect()
    }
}

prop_compose! {
    fn arb_server_data()(size in 0..50usize)
        (uuids in arb_replica_ids(),
        urls in prop::collection::hash_set("[a-zA-Z0-9-]+", size))
        -> Vec<(ReplicaUuid, String)> {
            uuids.into_iter().zip(urls.into_iter()).collect()
    }
}

// utility functions

/// Open an in-memory db for testing. If the environment variable `TKZR_TEST_SAVE_DB` is set, the
/// db used for this test is saved on disk into `/tmp/tkzr/test` and the directory is printed at runtime.
pub fn open_test_db() -> SqliteBackend {
    let res = match env::var("TKZR_TEST_SAVE_DB") {
        Ok(_) => {
            // the into_path() call "leaks" the tempdir so it is not deleted
            let test_dir = TempDir::new_in("/tmp/tkzr/test").expect("temporary test database directory could not be created")
                .into_path();

            println!("creating test db in: {}", test_dir.display());

            SqliteBackend::open(&test_dir)
        },
        Err(_) => SqliteBackend::open_in_memory(),
    };

    assert!(res.is_ok(), "Failed to open sqlite backend: {}", res.unwrap_err());
    res.unwrap()
}

/// Opens an on-disk database for testing
/// Returns the TempDir so that it does not go out of scope and delete the database file
pub fn open_test_db_on_disk() -> (SqliteBackend, TempDir) {
    let test_dir = tempdir().expect("temporary directory could not be created");
 
    let res = SqliteBackend::open(&test_dir);
    assert!(res.is_ok(), "Error opening on-disk sqlite backend: {}", res.unwrap_err());
    let db = res.unwrap();

    (db, test_dir)
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
    let dir = { // open connection
        let (_db_will_drop, dir) = open_test_db_on_disk();

        dir
    }; // close connection on drop and record time
    let after_creation = Utc::now();

    // open again and read metadata
    let mut db = SqliteBackend::open(&dir).expect("opening database failed");
    let tx = db.transaction().expect("Failed to begin transaction");
    let metadata = tx.metadata().expect("getting db metadata failed");

    let ver = env!("CARGO_PKG_VERSION");

    assert!(metadata.version == ver, "Versions do not match: db version {}, crate version {}", metadata.version, ver);
    assert!(metadata.date_created >= before_creation, "Database was created in the past");
    assert!(metadata.date_created < after_creation, "Database was created in the future");
}
