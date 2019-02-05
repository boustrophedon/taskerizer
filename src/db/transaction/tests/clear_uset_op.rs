use rand::prelude::*;

use crate::db::DBTransaction;
use crate::db::tests::open_test_db;

use crate::sync::USetOpMsg;
use crate::sync::test_utils::{example_add_uset_op_1, example_remove_uset_op_1,
                              example_add_uset_op_2,
                              uset_add_list_arb};
use crate::sync::test_utils::{example_client_1, example_client_2, example_client_3};

#[test]
/// Clear client with no msgs, check result is okay.
fn test_tx_clear_uset_op_empty() {
    let deliver_to = example_client_1();

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let res = tx.clear_uset_op_msgs(&deliver_to);
    assert!(res.is_ok(), "Clearing uset msgs failed: {}", res.unwrap_err());
}

#[test]
/// Add one add message, clear messages, check no messages.
fn test_tx_clear_uset_op_one() {
    let deliver_to = example_client_1();
    let op = example_add_uset_op_1();

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let msg = USetOpMsg { op, deliver_to };
    tx.store_uset_op_msg(&msg).expect("Failed to store message");

    let res = tx.clear_uset_op_msgs(&deliver_to);
    assert!(res.is_ok(), "Error clearing uset msgs: {}", res.unwrap_err());

    let msgs = tx.fetch_uset_op_msgs(&deliver_to).expect("Failed to fetch messages");
    assert!(msgs.is_empty(), "Had messages for client after clearing: {:?}", msgs);
}

#[test]
/// Add one remove message, clear messages, check no messages.
fn test_tx_clear_uset_op_one_remove() {
    let deliver_to = example_client_1();
    let op = example_remove_uset_op_1();

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let msg = USetOpMsg { op, deliver_to };
    tx.store_uset_op_msg(&msg).expect("Failed to store message");

    let res = tx.clear_uset_op_msgs(&deliver_to);
    assert!(res.is_ok(), "Error clearing uset msgs: {}", res.unwrap_err());

    let msgs = tx.fetch_uset_op_msgs(&deliver_to).expect("Failed to fetch messages");
    assert!(msgs.is_empty(), "Had messages for client after clearing: {:?}", msgs);
}

#[test]
/// Add one message for two clients, clear messages on first, check no messages on first and one on
/// second.
fn test_tx_clear_uset_op_two_clients() {
    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    // message for client 1
    let deliver_to = example_client_1();
    let op = example_add_uset_op_1();
    let msg1 = USetOpMsg { op, deliver_to };
    tx.store_uset_op_msg(&msg1).expect("Failed to store message");

    // message for client 2
    let deliver_to = example_client_2();
    let op = example_add_uset_op_2();
    let msg2 = USetOpMsg { op, deliver_to };
    tx.store_uset_op_msg(&msg2).expect("Failed to store message");

    // clear messages for client 1
    let res = tx.clear_uset_op_msgs(&example_client_1());
    assert!(res.is_ok(), "Error clearing uset msgs: {}", res.unwrap_err());

    // assert no messages for client 1
    let msgs = tx.fetch_uset_op_msgs(&example_client_1()).expect("Failed to fetch messages");
    assert!(msgs.is_empty(), "Had messages for client after clearing: {:?}", msgs);

    // assert 1 message for client 2
    let msgs = tx.fetch_uset_op_msgs(&example_client_2()).expect("Failed to fetch messages");
    assert_eq!(msgs.len(), 1, "Incorrect number of messages for client 2: {:?}", msgs);

    // assert correct message for client 2
    assert_eq!(&msgs[0], &msg2, "Unexpected message for client 2")
}

proptest! {
    #[test]
    /// Store the same messages for 3 clients, clear each and check at each step the remaining clients still
    /// have their messages.
    fn test_tx_clear_uset_arb(adds in uset_add_list_arb(), removes in uset_add_list_arb()) {
        let mut db = open_test_db();
        let tx = db.transaction().unwrap();

        let mut messages = [Vec::new(), Vec::new(), Vec::new()];
        let clients = [example_client_1(), example_client_2(), example_client_3()];

        // put each set of messages into a per-client vector, and then shuffle them into a random
        // order.
        //
        // NOTE: these operations are random: they do not satisfy any sort of causal relationship
        // or other invariants required for the CRDT. we just are checking that fetch returns them
        // in the order they were added.

        let mut rng = thread_rng();
        for i in 0..3 {
            let deliver_to = clients[i];
            let add_msgs = adds.iter().cloned().map(|op| USetOpMsg { op, deliver_to });
            let remove_msgs = removes.iter().cloned().map(|op| op.into_remove()).map(|op| USetOpMsg { op, deliver_to });
 
            messages[i].extend(add_msgs);
            messages[i].extend(remove_msgs);
            messages[i].shuffle(&mut rng);
        }

        // store messages for each client
        for i in 0..3 {
            for msg in &messages[i] {
                tx.store_uset_op_msg(&msg).expect("Storing USetOpMsg failed");
            }
        }

        // clear first client's messages, check remaining two are still there
        let res = tx.clear_uset_op_msgs(&clients[0]);
        prop_assert!(res.is_ok(), "Error clearing uset operations for client {}: {}", 0, res.unwrap_err());

        // check no messages in first client's inbox
        let msgs = tx.fetch_uset_op_msgs(&clients[0]).unwrap();
        prop_assert!(msgs.is_empty(), "Cleared messages were not empty: {:?}", msgs);

        // check remaining client 1's messages are still there
        let msgs = tx.fetch_uset_op_msgs(&clients[1]).unwrap();
        prop_assert_eq!(&msgs, &messages[1]);
        // check remaining client 2's messages are still there
        let msgs = tx.fetch_uset_op_msgs(&clients[2]).unwrap();
        prop_assert_eq!(&msgs, &messages[2]);


        // clear second client's messages, check remaining client's are still there
        let res = tx.clear_uset_op_msgs(&clients[1]);
        prop_assert!(res.is_ok(), "Error clearing uset operations for client {}: {}", 1, res.unwrap_err());

        // check remaining client's messages are still there
        let msgs = tx.fetch_uset_op_msgs(&clients[2]).unwrap();
        prop_assert_eq!(&msgs, &messages[2]);


        // finally, clear last client's messages, check all three clients' messages are empty.
        let res = tx.clear_uset_op_msgs(&clients[2]);
        prop_assert!(res.is_ok(), "Error clearing uset operations for client {}: {}", 2, res.unwrap_err());

        // check first client's messages are (still) gone
        let msgs = tx.fetch_uset_op_msgs(&clients[0]).unwrap();
        prop_assert!(msgs.is_empty());

        // check second client's messages are (still) gone
        let msgs = tx.fetch_uset_op_msgs(&clients[1]).unwrap();
        prop_assert!(msgs.is_empty());

        // check third client's messages are gone
        let msgs = tx.fetch_uset_op_msgs(&clients[2]).unwrap();
        prop_assert!(msgs.is_empty());
    }
}
