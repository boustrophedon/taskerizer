use rand::prelude::*;

use crate::db::DBTransaction;
use crate::db::tests::open_test_db;

use crate::sync::USetOpMsg;
use crate::sync::test_utils::{example_add_uset_op_1, example_remove_uset_op_1,
                              example_add_uset_op_2, example_remove_uset_op_2,
                              uset_add_list_arb};
use crate::sync::test_utils::{example_replica_1, example_replica_2, example_replica_3};

#[test]
/// Fetch replica with no msgs, check no results are returned.
fn test_tx_fetch_uset_op_empty() {
    let deliver_to = example_replica_1();

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let res = tx.fetch_uset_op_msgs(&deliver_to);
    assert!(res.is_ok(), "Fetching uset add op msg failed: {}", res.unwrap_err());

    let msgs = res.unwrap();
    assert_eq!(msgs.len(), 0, "Incorrect number of messages returned from fetch. {:?}", msgs);
}

#[test]
/// Store uset op for one replica, fetch different replica with no msgs, check no results are returned.
fn test_tx_fetch_uset_op_empty_other() {
    let op = example_add_uset_op_1();
    let deliver_to = example_replica_1();
    let msg = USetOpMsg { op, deliver_to };

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    tx.store_uset_op_msg(&msg).expect("Failed to store uset msg");

    let res = tx.fetch_uset_op_msgs(&example_replica_2());
    assert!(res.is_ok(), "Fetching uset add op msg failed: {}", res.unwrap_err());

    let msgs = res.unwrap();
    assert_eq!(msgs.len(), 0, "Incorrect number of messages returned from fetch. {:?}", msgs);
}

#[test]
/// Store add USetOpMsg, fetch it, check it's the same.
fn test_tx_fetch_uset_op_add() {
    let op = example_add_uset_op_1();
    let deliver_to = example_replica_1();
    let msg = USetOpMsg { op, deliver_to };

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    tx.store_uset_op_msg(&msg).expect("Failed to store uset msg");

    let res = tx.fetch_uset_op_msgs(&deliver_to);
    assert!(res.is_ok(), "Fetching uset add op msg failed: {}", res.unwrap_err());

    let msgs = res.unwrap();
    assert_eq!(msgs.len(), 1, "Incorrect number of messages returned from fetch. {:?}", msgs);
    assert_eq!(msgs[0], msg);
}

#[test]
/// Store remove USetOpMsg, fetch it, check it's the same.
fn test_tx_fetch_uset_op_remove() {
    let op = example_remove_uset_op_1();
    let deliver_to = example_replica_1();
    let msg = USetOpMsg { op, deliver_to };

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    tx.store_uset_op_msg(&msg).expect("Failed to store uset msg");

    let res = tx.fetch_uset_op_msgs(&deliver_to);
    assert!(res.is_ok(), "Fetching uset add op msg failed: {}", res.unwrap_err());

    let msgs = res.unwrap();
    assert_eq!(msgs.len(), 1, "Incorrect number of messages returned from fetch. {:?}", msgs);
    assert_eq!(msgs[0], msg);
}


#[test]
/// Store add USetOpMsgs for 2 replicas, fetch for each and check we only get their respective messages.
fn test_tx_fetch_uset_op_add_multi() {
    let op = example_add_uset_op_1();
    let deliver_to = example_replica_1();
    let msg1 = USetOpMsg { op, deliver_to };

    let op = example_add_uset_op_2();
    let deliver_to = example_replica_2();
    let msg2 = USetOpMsg { op, deliver_to };

    let op = example_remove_uset_op_2();
    let deliver_to = example_replica_2();
    let msg3 = USetOpMsg { op, deliver_to };

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    tx.store_uset_op_msg(&msg1).expect("Failed to store uset op msg");
    tx.store_uset_op_msg(&msg2).expect("Failed to store uset op msg");
    tx.store_uset_op_msg(&msg3).expect("Failed to store uset op msg");

    // check messages for replica 1
    let res = tx.fetch_uset_op_msgs(&example_replica_1());
    assert!(res.is_ok(), "Fetching uset add op msg failed: {}", res.unwrap_err());

    let msgs = res.unwrap();
    assert_eq!(msgs.len(), 1, "Incorrect number of messages returned from fetch. {:?}", msgs);
    assert_eq!(msgs[0], msg1);

    // check messages for replica 2
    let res = tx.fetch_uset_op_msgs(&example_replica_2());
    assert!(res.is_ok(), "Fetching uset add op msg failed: {}", res.unwrap_err());

    let msgs = res.unwrap();
    assert_eq!(msgs.len(), 2, "Incorrect number of messages returned from fetch. {:?}", msgs);
    assert_eq!(msgs[0], msg2);
    assert_eq!(msgs[1], msg3);
}

use proptest::array::uniform3;

proptest! {
    #[test]
    /// Store and fetch adds and removes for 3 replicas, make sure each fetch gets the correct msgs
    /// back in the correct order.
    fn test_tx_fetch_uset_arb(mut adds in uniform3(uset_add_list_arb()), mut removes in uniform3(uset_add_list_arb())) {
        let mut db = open_test_db();
        let tx = db.transaction().unwrap();

        let mut messages = [Vec::new(), Vec::new(), Vec::new()];
        let replicas = [example_replica_1(), example_replica_2(), example_replica_3()];

        // put each set of messages into a per-replica vector, and then shuffle them into a random
        // order.
        //
        // NOTE: these operations are random: they do not satisfy any sort of causal relationship
        // or other invariants required for the CRDT. we just are checking that fetch returns them
        // in the order they were added.

        let mut rng = thread_rng();
        for i in 0..3 {
            let deliver_to = replicas[i];
            let add_msgs = adds[i].drain(..).map(|op| USetOpMsg { op, deliver_to });
            let remove_msgs = removes[i].drain(..).map(|op| op.into_remove()).map(|op| USetOpMsg { op, deliver_to });
            
            messages[i].extend(add_msgs);
            messages[i].extend(remove_msgs);
            messages[i].shuffle(&mut rng);
        }

        for i in 0..3 {
            for msg in &messages[i] {
                tx.store_uset_op_msg(&msg).expect("Storing USetOpMsg failed");
            }
        }

        for i in 0..3 {
            let res = tx.fetch_uset_op_msgs(&replicas[i]);
            prop_assert!(res.is_ok(), "Error fetching uset operations for replica {}: {}", i, res.unwrap_err());

            let msgs = res.unwrap();
            prop_assert_eq!(msgs.len(), messages[i].len(), "Incorrect number of messages returned from fetch. {:?}", msgs);
            prop_assert_eq!(&msgs, &messages[i]);
        }
    }
}
