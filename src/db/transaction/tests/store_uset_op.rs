use uuid::Uuid;

use crate::db::DBTransaction;
use crate::db::tests::open_test_db;

use crate::sync::USetOpMsg;
use crate::sync::test_utils::{example_add_uset_op_1, example_remove_uset_op_1, example_remove_uset_op_2, uset_add_list_arb};
use crate::sync::test_utils::{example_client_1, example_client_2};

#[test]
fn test_tx_store_uset_op_add() {
    let op = example_add_uset_op_1();
    let deliver_to = example_client_1();
    let msg = USetOpMsg { op, deliver_to };

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let res = tx.store_uset_op_msg(&msg);
    assert!(res.is_ok(), "Storing uset add op msg failed: {}", res.unwrap_err());
}

#[test]
fn test_tx_store_uset_op_remove() {
    let op = example_remove_uset_op_1();
    let deliver_to = example_client_1();
    let msg = USetOpMsg { op, deliver_to };

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let res = tx.store_uset_op_msg(&msg);
    assert!(res.is_ok(), "Storing uset remove op msg failed: {}", res.unwrap_err());
}

#[test]
/// Storing duplicate duplicate remove messages is allowed.
fn test_tx_store_uset_op_duplicates_ok() {
    let op = example_remove_uset_op_1();
    let deliver_to = example_client_1();
    let msg1 = USetOpMsg { op, deliver_to };

    let op = example_remove_uset_op_2();
    let deliver_to = example_client_2();
    let msg2 = USetOpMsg { op, deliver_to };

    let mut db = open_test_db();
    let tx = db.transaction().unwrap();

    let res = tx.store_uset_op_msg(&msg1);
    assert!(res.is_ok(), "Storing uset remove op msg failed: {}", res.unwrap_err());
    let res = tx.store_uset_op_msg(&msg1);
    assert!(res.is_ok(), "Storing uset remove op msg twice failed: {}", res.unwrap_err());

    let res = tx.store_uset_op_msg(&msg2);
    assert!(res.is_ok(), "Storing uset remove op msg failed: {}", res.unwrap_err());
    let res = tx.store_uset_op_msg(&msg2);
    assert!(res.is_ok(), "Storing uset remove op msg twice failed: {}", res.unwrap_err());
}

proptest! {
    #[test]
    fn test_tx_store_uset_arb(adds in uset_add_list_arb(), removes in uset_add_list_arb()) {
        let mut db = open_test_db();
        let tx = db.transaction().unwrap();

        for op in adds {
            let deliver_to = Uuid::new_v4();
            let msg = USetOpMsg { op, deliver_to };

            let res = tx.store_uset_op_msg(&msg);
            prop_assert!(res.is_ok(), "Storing USet add op failed: {}", res.unwrap_err());
        }

        for op in removes {
            let op = op.into_remove();
            let deliver_to = Uuid::new_v4();
            let msg = USetOpMsg { op, deliver_to };

            let res = tx.store_uset_op_msg(&msg);
            prop_assert!(res.is_ok(), "Storing USet remove op failed: {}", res.unwrap_err());
        }
    }
}
