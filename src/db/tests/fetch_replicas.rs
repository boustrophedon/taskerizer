use crate::db::DBBackend;
use crate::db::tests::open_test_db;
use crate::sync::ReplicaUuid;

use crate::sync::test_utils::{example_replica_1, example_replica_2};
use super::{arb_server_data, arb_replica_ids};

use std::collections::{HashSet, HashMap};

// the v1/v2 things don't mean anything: they're just examples.
const EXAMPLE_API_URL: &'static str = "http://api.example.com/tkzr/v1/";
const EXAMPLE_API_URL2: &'static str = "http://test-server.test/api/v2/";

#[test]
/// Fetch replicas with 1 server and 1 client replica
fn test_tx_fetch_replica_client_and_server() {
    let mut db = open_test_db();
    let mut tx = db.transaction().unwrap();

    let replica_id = example_replica_1();
    tx.store_replica_server(&replica_id, EXAMPLE_API_URL).expect("failed to store server");
    let replica_id2 = example_replica_2();
    tx.store_replica_client(&replica_id2).expect("failed to store client replica");

    let res = tx.fetch_replicas();
    assert!(res.is_ok(), "Failed to fetch replicas: {}", res.unwrap_err());

    let replicas = res.unwrap();
    assert_eq!(replicas.len(), 2, "Incorrect number of replicas fetched from db");
    assert!(replicas.contains(&(replica_id, Some(EXAMPLE_API_URL.into()))));
    assert!(replicas.contains(&(replica_id2, None)));
}

#[test]
/// Fetch replicas from empty db and check that we get an empty vec
fn test_tx_fetch_replica_empty() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let res = tx.fetch_replicas();
    assert!(res.is_ok(), "Failed to fetch servers: {}", res.unwrap_err());

    let servers = res.unwrap();
    assert!(servers.is_empty(), "Servers found in empty db: {:?}", servers);
}

#[test]
/// Add server with example url, fetch it and check that the server is there
fn test_tx_fetch_replica_server() {
    let mut db = open_test_db();
    let mut tx = db.transaction().unwrap();


    let replica_id = example_replica_1();
    tx.store_replica_server(&replica_id, EXAMPLE_API_URL).expect("failed to store server");

    let res = tx.fetch_replicas();
    assert!(res.is_ok(), "Failed to fetch servers: {}", res.unwrap_err());

    // turn Vec<(ReplicaUuid, Option<String>)> into Vec<(ReplicaUuid, String)>
    let servers: Vec<(ReplicaUuid, String)> = res.unwrap().into_iter()
        .map(|(uuid, opt_api)| (uuid, opt_api.expect("Expected api url")))
        .collect();
    assert_eq!(servers.len(), 1, "Incorrect number of servers fetch from db");
    assert_eq!(servers, vec![(replica_id, EXAMPLE_API_URL.into()),]);
}

#[test]
/// Add two servers with example urls, fetch and check they are as expected
fn test_tx_fetch_replica_server_2() {
    let mut db = open_test_db();
    let mut tx = db.transaction().unwrap();


    let replica_id = example_replica_1();
    tx.store_replica_server(&replica_id, EXAMPLE_API_URL).expect("failed to store server");

    let replica_id2 = example_replica_2();
    tx.store_replica_server(&replica_id2, EXAMPLE_API_URL2).expect("failed to store server");

    let res = tx.fetch_replicas();
    assert!(res.is_ok(), "Failed to fetch servers: {}", res.unwrap_err());

    // turn Vec<(ReplicaUuid, Option<String>)> into Vec<(ReplicaUuid, String)>
    let servers: Vec<(ReplicaUuid, String)> = res.unwrap().into_iter()
        .map(|(uuid, opt_api)| (uuid, opt_api.expect("Expected api url")))
        .collect();
    assert_eq!(servers.len(), 2, "Incorrect number of servers fetch from db");
    assert_eq!(servers, vec![(replica_id, EXAMPLE_API_URL.into()), (replica_id2, EXAMPLE_API_URL2.into())]);
}

#[test]
/// Add two servers with non-unique urls, check that we get an error when adding the second and the
/// db has the first entry.
/// https://www.sqlite.org/lang_conflict.html says that by default (and in the sql standard) only
/// the error-causing statement is aborted and the transaction remains active.
fn test_tx_fetch_replica_server_duplicate_url() {
    let mut db = open_test_db();
    let mut tx = db.transaction().unwrap();


    let replica_id = example_replica_1();
    tx.store_replica_server(&replica_id, EXAMPLE_API_URL).expect("failed to store server");
    let replica_id2 = example_replica_2();
    tx.store_replica_server(&replica_id2, EXAMPLE_API_URL).expect_err("successfully stored duplicate server api url");

    tx.finish().expect("committing transaction failed");

    // now check only the first server made it in
    let tx = db.transaction().unwrap();
    let res = tx.fetch_replicas();
    assert!(res.is_ok(), "Failed to fetch servers: {}", res.unwrap_err());

    println!("{:?}", res);
    // turn Vec<(ReplicaUuid, Option<String>)> into Vec<(ReplicaUuid, String)>
    let servers: Vec<(ReplicaUuid, String)> = res.unwrap().into_iter()
        .map(|(uuid, opt_api)| (uuid, opt_api.expect("Expected api url")))
        .collect();
    assert_eq!(servers.len(), 1, "Incorrect number of servers fetch from db");
    assert_eq!(servers, vec![(replica_id, EXAMPLE_API_URL.into()),]);

}

#[test]
/// Add two servers with non-unique replica ids, check that we get an error when adding the second
fn test_tx_fetch_replica_server_duplicate_replica_id() {
    let mut db = open_test_db();
    let mut tx = db.transaction().unwrap();


    let replica_id = example_replica_1();
    tx.store_replica_server(&replica_id, EXAMPLE_API_URL).expect("failed to store server");
    tx.store_replica_server(&replica_id, EXAMPLE_API_URL2).expect_err("successfully stored duplicate replica id");

    tx.finish().expect("committing transaction failed");

    // now check only the first server made it in
    let tx = db.transaction().unwrap();
    let res = tx.fetch_replicas();
    assert!(res.is_ok(), "Failed to fetch servers: {}", res.unwrap_err());

    // turn Vec<(ReplicaUuid, Option<String>)> into Vec<(ReplicaUuid, String)>
    let servers: Vec<(ReplicaUuid, String)> = res.unwrap().into_iter()
        .map(|(uuid, opt_api)| (uuid, opt_api.expect("Expected api url")))
        .collect();
    assert_eq!(servers.len(), 1, "Incorrect number of servers fetched frm db");
    assert_eq!(servers, vec![(replica_id, EXAMPLE_API_URL.into()),]);

}

proptest! {
    #[test]
    fn test_tx_fetch_replica_server_arb(data in arb_server_data()) {
        let mut db = open_test_db();
        let mut tx = db.transaction().unwrap();

        for (replica_id, url) in &data {
            tx.store_replica_server(replica_id, url).expect("Failed to store server");
        }

        let res = tx.fetch_replicas();
        prop_assert!(res.is_ok(), "Failed to fetch servers: {}", res.unwrap_err());

        // turn Vec<(ReplicaUuid, Option<String>)> into Vec<(ReplicaUuid, String)>
        let servers: Vec<(ReplicaUuid, String)> = res.unwrap().into_iter()
            .map(|(uuid, opt_api)| (uuid, opt_api.expect("Expected api url")))
            .collect();
        prop_assert_eq!(servers.len(), data.len(), "Incorrect number of servers fetched from db");
        prop_assert_eq!(servers, data);
    }
}

proptest! {
    #[test]
    fn test_tx_fetch_replica_client_arb(replica_ids in arb_replica_ids()) {
        let mut db = open_test_db();
        let tx = db.transaction().unwrap();

        for replica_id in &replica_ids {
            tx.store_replica_client(replica_id).expect("Failed to store client");
        }

        let res = tx.fetch_replicas();
        prop_assert!(res.is_ok(), "Failed to fetch replicas: {}", res.unwrap_err());

        // turn Vec<(ReplicaUuid, Option<String>)> into Vec<ReplicaUuid>
        // asserting that all options are None
        let clients: Vec<ReplicaUuid> = res.unwrap().into_iter()
            .map(|(uuid, opt_api)| {
                assert!(opt_api.is_none());
                uuid
            })
            .collect();
        prop_assert_eq!(clients.len(), replica_ids.len(), "Incorrect number of clients fetched from db");
        prop_assert_eq!(clients, replica_ids);
    }
}

proptest! {
    #[test]
    fn test_tx_fetch_replicas_arb(replica_ids in arb_replica_ids(), data in arb_server_data()) {
        let mut db = open_test_db();
        let mut tx = db.transaction().unwrap();

        let mut server_map = HashMap::new();
        for (replica_id, url) in &data {
            tx.store_replica_server(replica_id, url).expect("Failed to store server");
            server_map.insert(replica_id, url);
        }

        let mut clients_set = HashSet::new();
        for replica_id in &replica_ids {
            tx.store_replica_client(replica_id).expect("Failed to store client");
            clients_set.insert(replica_id);
        }

        let res = tx.fetch_replicas();
        prop_assert!(res.is_ok(), "Failed to fetch replicas: {}", res.unwrap_err());

        let replicas = res.unwrap();
        prop_assert_eq!(clients_set.len()+server_map.len(), replicas.len(), "Incorrect number of replicas fetched from db");

        for (replica_id, url_opt) in &replicas {
            match url_opt {
                Some(url) => {
                    // if we added a replica via store_replica_client, it shouldn't show up with an
                    // api url
                    prop_assert!(!clients_set.contains(replica_id));

                    prop_assert_eq!(url, server_map[replica_id]);
                }
                None => {
                    prop_assert!(!server_map.contains_key(replica_id));
                    prop_assert!(clients_set.contains(replica_id));
                }
            }
        }
    }
}
