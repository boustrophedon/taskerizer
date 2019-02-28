use crate::db::DBTransaction;
use crate::db::tests::open_test_db;

use crate::sync::test_utils::{example_replica_1, example_replica_2};
use super::arb_replica_ids;

#[test]
/// Add client
fn test_tx_store_replica() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let replica_id = example_replica_1();
    let res = tx.store_replica(&replica_id);
    assert!(res.is_ok(), "Error adding replica: {}", res.unwrap_err());
}

#[test]
/// Add two clients
fn test_tx_store_replica_2() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let replica_id = example_replica_1();
    let res = tx.store_replica(&replica_id);
    assert!(res.is_ok(), "Error adding replica: {}", res.unwrap_err());

    let replica_id = example_replica_2();
    let res = tx.store_replica(&replica_id);
    assert!(res.is_ok(), "Error adding second replica: {}", res.unwrap_err());
}

#[test]
/// Add two replicas with non-unique replica ids, check that we get an error when adding the second
fn test_tx_store_replica_duplicate_replica_id() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();


    let replica_id = example_replica_1();
    let res = tx.store_replica(&replica_id);
    assert!(res.is_ok(), "Error adding replica: {}", res.unwrap_err());

    let replica_id = example_replica_1();
    let res = tx.store_replica(&replica_id);
    assert!(res.is_err(), "Got ok when adding same replica id twice");

    let err = res.unwrap_err();
    assert!(err.to_string().contains("UNIQUE constraint failed: replicas.replica_uuid"), "Incorrect error message, expected unique violation got: {}", err);
}

proptest! {
    #[test]
    fn test_tx_store_replica_arb(replica_ids in arb_replica_ids()) {
        let mut db = open_test_db();
        let tx = db.transaction().unwrap();

        for replica_id in &replica_ids {
            let res = tx.store_replica(replica_id);
            assert!(res.is_ok(), "Error adding replica: {}", res.unwrap_err());
        }
    }
}
