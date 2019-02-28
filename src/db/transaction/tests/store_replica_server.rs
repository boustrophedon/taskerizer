use crate::db::DBTransaction;
use crate::db::tests::open_test_db;

use crate::sync::test_utils::{example_replica_1, example_replica_2};
use super::arb_server_data;

// the v1/v2 things don't mean anything: they're just examples.
const EXAMPLE_API_URL: &'static str = "http://api.example.com/tkzr/v1/";
const EXAMPLE_API_URL2: &'static str = "http://test-server.test/api/v2/";

#[test]
/// Add server with example url
fn test_tx_store_replica_server() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();


    let replica_id = example_replica_1();
    let res = tx.store_replica_server(EXAMPLE_API_URL, &replica_id);
    assert!(res.is_ok(), "Error adding replica server: {}", res.unwrap_err());
}

#[test]
/// Add two servers with example urls
fn test_tx_store_replica_server_2() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();


    let replica_id = example_replica_1();
    let res = tx.store_replica_server(EXAMPLE_API_URL, &replica_id);
    assert!(res.is_ok(), "Error adding replica server: {}", res.unwrap_err());

    let replica_id = example_replica_2();
    let res = tx.store_replica_server(EXAMPLE_API_URL2, &replica_id);
    assert!(res.is_ok(), "Error adding second replica server: {}", res.unwrap_err());
}

#[test]
/// Add two servers with non-unique urls, check that we get an error when adding the second
fn test_tx_store_replica_server_duplicate_url() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();


    let replica_id = example_replica_1();
    let res = tx.store_replica_server(EXAMPLE_API_URL, &replica_id);
    assert!(res.is_ok(), "Error adding replica server: {}", res.unwrap_err());

    let replica_id = example_replica_2();
    let res = tx.store_replica_server(EXAMPLE_API_URL, &replica_id);
    assert!(res.is_err(), "Got ok when adding same server url twice");

    let err = res.unwrap_err();
    assert!(err.to_string().contains("UNIQUE constraint failed: servers.api_url"), "Incorrect error message, expected unique violation got: {}", err);
}

#[test]
/// Add two servers with non-unique replica ids, check that we get an error when adding the second
fn test_tx_store_replica_server_duplicate_replica_id() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();


    let replica_id = example_replica_1();
    let res = tx.store_replica_server(EXAMPLE_API_URL, &replica_id);
    assert!(res.is_ok(), "Error adding replica server: {}", res.unwrap_err());

    let replica_id = example_replica_1();
    let res = tx.store_replica_server(EXAMPLE_API_URL2, &replica_id);
    assert!(res.is_err(), "Got ok when adding same server replica id twice");

    let err = res.unwrap_err();
    assert!(err.to_string().contains("UNIQUE constraint failed: replicas.replica_uuid"), "Incorrect error message, expected unique violation got: {}", err);
}

proptest! {
    #[test]
    fn test_tx_store_replica_server_arb(data in arb_server_data()) {
        let mut db = open_test_db();
        let tx = db.transaction().unwrap();

        for (url, replica_id) in &data {
            let res = tx.store_replica_server(url, replica_id);
            assert!(res.is_ok(), "Error adding replica server: {}", res.unwrap_err());
        }
    }
}
