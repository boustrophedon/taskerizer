use rand::prelude::*;

use crate::db::DBTransaction;
use crate::db::tests::open_test_db;

use crate::sync::USetOpMsg;
use crate::sync::test_utils::{example_add_uset_op_1, example_remove_uset_op_1,
                              example_add_uset_op_2,
                              uset_add_list_arb};
use crate::sync::test_utils::{example_replica_1, example_replica_2, example_replica_3};

#[test]
/// Clear replica with no msgs, check result is okay.
fn test_tx_clear_uset_op_empty() {
    let deliver_to = example_replica_1();

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let res = tx.clear_uset_op_msgs(&deliver_to);
    assert!(res.is_ok(), "Clearing uset msgs failed: {}", res.unwrap_err());
}

#[test]
/// Add one add message, clear messages, check no messages.
fn test_tx_clear_uset_op_one() {
    let deliver_to = example_replica_1();
    let op = example_add_uset_op_1();

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let msg = USetOpMsg { op, deliver_to };
    tx.store_uset_op_msg(&msg).expect("Failed to store message");

    let res = tx.clear_uset_op_msgs(&deliver_to);
    assert!(res.is_ok(), "Error clearing uset msgs: {}", res.unwrap_err());

    let msgs = tx.fetch_uset_op_msgs(&deliver_to).expect("Failed to fetch messages");
    assert!(msgs.is_empty(), "Had messages for replica after clearing: {:?}", msgs);
}

#[test]
/// Add one remove message, clear messages, check no messages.
fn test_tx_clear_uset_op_one_remove() {
    let deliver_to = example_replica_1();
    let op = example_remove_uset_op_1();

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let msg = USetOpMsg { op, deliver_to };
    tx.store_uset_op_msg(&msg).expect("Failed to store message");

    let res = tx.clear_uset_op_msgs(&deliver_to);
    assert!(res.is_ok(), "Error clearing uset msgs: {}", res.unwrap_err());

    let msgs = tx.fetch_uset_op_msgs(&deliver_to).expect("Failed to fetch messages");
    assert!(msgs.is_empty(), "Had messages for replica after clearing: {:?}", msgs);
}

#[test]
/// Add one message for two replicas, clear messages on first, check no messages on first and one on
/// second.
fn test_tx_clear_uset_op_two_replicas() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    // message for replica 1
    let deliver_to = example_replica_1();
    let op = example_add_uset_op_1();
    let msg1 = USetOpMsg { op, deliver_to };
    tx.store_uset_op_msg(&msg1).expect("Failed to store message");

    // message for replica 2
    let deliver_to = example_replica_2();
    let op = example_add_uset_op_2();
    let msg2 = USetOpMsg { op, deliver_to };
    tx.store_uset_op_msg(&msg2).expect("Failed to store message");

    // clear messages for replica 1
    let res = tx.clear_uset_op_msgs(&example_replica_1());
    assert!(res.is_ok(), "Error clearing uset msgs: {}", res.unwrap_err());

    // assert no messages for replica 1
    let msgs = tx.fetch_uset_op_msgs(&example_replica_1()).expect("Failed to fetch messages");
    assert!(msgs.is_empty(), "Had messages for replica after clearing: {:?}", msgs);

    // assert 1 message for replica 2
    let msgs = tx.fetch_uset_op_msgs(&example_replica_2()).expect("Failed to fetch messages");
    assert_eq!(msgs.len(), 1, "Incorrect number of messages for replica 2: {:?}", msgs);

    // assert correct message for replica 2
    assert_eq!(&msgs[0], &msg2, "Unexpected message for replica 2")
}

proptest! {
    #[test]
    /// Store the same messages for 3 replicas, clear each and check at each step the remaining replicas still
    /// have their messages.
    fn test_tx_clear_uset_arb(adds in uset_add_list_arb(), removes in uset_add_list_arb()) {
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
            let add_msgs = adds.iter().cloned().map(|op| USetOpMsg { op, deliver_to });
            let remove_msgs = removes.iter().cloned().map(|op| op.into_remove()).map(|op| USetOpMsg { op, deliver_to });
 
            messages[i].extend(add_msgs);
            messages[i].extend(remove_msgs);
            messages[i].shuffle(&mut rng);
        }

        // store messages for each replica
        for i in 0..3 {
            for msg in &messages[i] {
                tx.store_uset_op_msg(&msg).expect("Storing USetOpMsg failed");
            }
        }

        // clear first replica's messages, check remaining two are still there
        let res = tx.clear_uset_op_msgs(&replicas[0]);
        prop_assert!(res.is_ok(), "Error clearing uset operations for replica {}: {}", 0, res.unwrap_err());

        // check no messages in first replica's inbox
        let msgs = tx.fetch_uset_op_msgs(&replicas[0]).unwrap();
        prop_assert!(msgs.is_empty(), "Cleared messages were not empty: {:?}", msgs);

        // check remaining replica 1's messages are still there
        let msgs = tx.fetch_uset_op_msgs(&replicas[1]).unwrap();
        prop_assert_eq!(&msgs, &messages[1]);
        // check remaining replica 2's messages are still there
        let msgs = tx.fetch_uset_op_msgs(&replicas[2]).unwrap();
        prop_assert_eq!(&msgs, &messages[2]);


        // clear second replica's messages, check remaining replica's are still there
        let res = tx.clear_uset_op_msgs(&replicas[1]);
        prop_assert!(res.is_ok(), "Error clearing uset operations for replica {}: {}", 1, res.unwrap_err());

        // check remaining replica's messages are still there
        let msgs = tx.fetch_uset_op_msgs(&replicas[2]).unwrap();
        prop_assert_eq!(&msgs, &messages[2]);


        // finally, clear last replica's messages, check all three replicas' messages are empty.
        let res = tx.clear_uset_op_msgs(&replicas[2]);
        prop_assert!(res.is_ok(), "Error clearing uset operations for replica {}: {}", 2, res.unwrap_err());

        // check first replica's messages are (still) gone
        let msgs = tx.fetch_uset_op_msgs(&replicas[0]).unwrap();
        prop_assert!(msgs.is_empty());

        // check second replica's messages are (still) gone
        let msgs = tx.fetch_uset_op_msgs(&replicas[1]).unwrap();
        prop_assert!(msgs.is_empty());

        // check third replica's messages are gone
        let msgs = tx.fetch_uset_op_msgs(&replicas[2]).unwrap();
        prop_assert!(msgs.is_empty());
    }
}
