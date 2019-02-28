use crate::db::DBTransaction;
use crate::db::tests::open_test_db;

use crate::sync::test_utils::{example_replica_1, example_replica_2};
use super::arb_server_data;

// the v1/v2 things don't mean anything: they're just examples.
const EXAMPLE_API_URL: &'static str = "http://api.example.com/tkzr/v1/";
const EXAMPLE_API_URL2: &'static str = "http://test-server.test/api/v2/";

#[test]
/// Fetch servers from empty db and check that we get an empty vec
fn test_tx_fetch_replica_server_empty() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let res = tx.fetch_replica_servers();
    assert!(res.is_ok(), "Failed to fetch servers: {}", res.unwrap_err());

    let servers = res.unwrap();
    assert!(servers.is_empty(), "Servers found in empty db: {:?}", servers);
}

#[test]
/// Add server with example url, fetch it and check that the server is there
fn test_tx_fetch_replica_server() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();


    let replica_id = example_replica_1();
    tx.store_replica_server(EXAMPLE_API_URL, &replica_id).expect("failed to store server");

    let res = tx.fetch_replica_servers();
    assert!(res.is_ok(), "Failed to fetch servers: {}", res.unwrap_err());

    let servers = res.unwrap();
    assert_eq!(servers.len(), 1, "Incorrect number of servers fetch from db");
    assert_eq!(servers, vec![(EXAMPLE_API_URL.into(), replica_id),]);
}

#[test]
/// Add two servers with example urls, fetch and check they are as expected
fn test_tx_fetch_replica_server_2() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();


    let replica_id = example_replica_1();
    tx.store_replica_server(EXAMPLE_API_URL, &replica_id).expect("failed to store server");

    let replica_id2 = example_replica_2();
    tx.store_replica_server(EXAMPLE_API_URL2, &replica_id2).expect("failed to store server");

    let res = tx.fetch_replica_servers();
    assert!(res.is_ok(), "Failed to fetch servers: {}", res.unwrap_err());

    let servers = res.unwrap();
    assert_eq!(servers.len(), 2, "Incorrect number of servers fetch from db");
    assert_eq!(servers, vec![(EXAMPLE_API_URL.into(), replica_id), (EXAMPLE_API_URL2.into(), replica_id2)]);
}

#[test]
/// Add two servers with non-unique urls, check that we get an error when adding the second and the
/// db has the first entry.
/// https://www.sqlite.org/lang_conflict.html says that by default (and in the sql standard) only
/// the error-causing statement is aborted and the transaction remains active.
fn test_tx_fetch_replica_server_duplicate_url() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();


    let replica_id = example_replica_1();
    tx.store_replica_server(EXAMPLE_API_URL, &replica_id).expect("failed to store server");
    let replica_id2 = example_replica_2();
    tx.store_replica_server(EXAMPLE_API_URL, &replica_id2).expect_err("successfully stored duplicate server api url");

    tx.commit().expect("committing transaction failed");

    // now check only the first server made it in
    let tx = db.transaction().unwrap();
    let res = tx.fetch_replica_servers();
    assert!(res.is_ok(), "Failed to fetch servers: {}", res.unwrap_err());

    let servers = res.unwrap();
    assert_eq!(servers.len(), 1, "Incorrect number of servers fetch from db");
    assert_eq!(servers, vec![(EXAMPLE_API_URL.into(), replica_id),]);

}

#[test]
/// Add two servers with non-unique replica ids, check that we get an error when adding the second
fn test_tx_fetch_replica_server_duplicate_replica_id() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();


    let replica_id = example_replica_1();
    tx.store_replica_server(EXAMPLE_API_URL, &replica_id).expect("failed to store server");
    tx.store_replica_server(EXAMPLE_API_URL2, &replica_id).expect_err("successfully stored duplicate replica id");

    tx.commit().expect("committing transaction failed");

    // now check only the first server made it in
    let tx = db.transaction().unwrap();
    let res = tx.fetch_replica_servers();
    assert!(res.is_ok(), "Failed to fetch servers: {}", res.unwrap_err());

    let servers = res.unwrap();
    assert_eq!(servers.len(), 1, "Incorrect number of servers fetch from db");
    assert_eq!(servers, vec![(EXAMPLE_API_URL.into(), replica_id),]);

}

proptest! {
    #[test]
    fn test_tx_fetch_replica_server_arb(data in arb_server_data()) {
        let mut db = open_test_db();
        let tx = db.transaction().unwrap();

        for (url, replica_id) in &data {
            tx.store_replica_server(url, replica_id).expect("Failed to store server");
        }

        let res = tx.fetch_replica_servers();
        prop_assert!(res.is_ok(), "Failed to fetch servers: {}", res.unwrap_err());

        let servers = res.unwrap();
        prop_assert_eq!(servers.len(), data.len(), "Incorrect number of servers fetch from db");
        prop_assert_eq!(servers, data);
    }
}
