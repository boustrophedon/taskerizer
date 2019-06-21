use crate::db::DBBackend;
use crate::db::tests::open_test_db;

use crate::sync::USetOp;
use crate::sync::server::process_sync;

use crate::sync::test_utils::{example_replica_1, example_replica_2, example_replica_3};
use crate::sync::test_utils::{example_add_uset_op_1, example_add_uset_op_2, example_remove_uset_op_1};
use crate::sync::test_utils::uset_add_list_arb;

use crate::task::test_utils::example_task_list;

use pretty_assertions::assert_eq;

#[test]
/// If incoming replica is not in database, send nothing if no tasks are in database.
fn test_server_process_sync_empty() {
    let mut db = open_test_db();
    let mut tx = db.transaction().expect("Failed to open transaction");

    let inc_repl = example_replica_1();

    let res = process_sync(&mut tx, inc_repl, &[]);
    assert!(res.is_ok(), "Error processing empty incoming operations from new replica: {}", res.unwrap_err());

    let resp_ops = res.unwrap();
    assert!(resp_ops.is_empty(), "Ops in response from empty database: {:?}", resp_ops);
}

#[test]
/// If incoming replica is not in database, recieve ops, send nothing in response if no tasks are
/// in database, and then send nothing again if we try to sync again.
fn test_server_process_sync_empty_send_ops() {
    let mut db = open_test_db();
    let mut tx = db.transaction().expect("Failed to open transaction");

    let inc_repl = example_replica_1();
    let ops = vec![example_add_uset_op_1(), example_remove_uset_op_1()];

    // process incoming ops, returns nothing
    let res = process_sync(&mut tx, inc_repl, &ops);
    assert!(res.is_ok(), "Error processing incoming operations from new replica: {}", res.unwrap_err());

    let resp_ops = res.unwrap();
    assert!(resp_ops.is_empty(), "Ops in response from empty database: {:?}", resp_ops);

    // client syncs, make sure no ops in response
    let res = process_sync(&mut tx, inc_repl, &[]);
    assert!(res.is_ok(), "Error processing empty incoming operations from new replica: {}", res.unwrap_err());

    let resp_ops = res.unwrap();
    assert!(resp_ops.is_empty(), "Ops in response after syncing twice with no operations in between: {:?}", resp_ops);
}

// FIXME: the fact that these are two separate tests means we should do some refactoring.

#[test]
/// On initial sync, send same task/operation twice and check we get an error
fn test_server_process_sync_error_on_duplicate_initial() {
    let mut db = open_test_db();
    let mut tx = db.transaction().expect("Failed to open transaction");

    let replica = example_replica_1();
    let ops = vec![example_add_uset_op_1(), example_add_uset_op_1()];

    // try to process same op twice, fail
    let res = process_sync(&mut tx, replica, &ops);
    assert!(res.is_err(), "No error when processing same operation twice: {:?}", res.unwrap());

    let err = res.unwrap_err();
    assert!(err.to_string().contains("Failed to apply incoming operations"), "Error message was incorrect: {}", err);
    assert!(err.to_string().contains("UNIQUE constraint failed: tasks.uuid"), "Error message was incorrect: {}", err);
}

#[test]
/// With registered client, send same task/operation twice and check we get an error
fn test_server_process_sync_error_on_duplicate_registered() {
    let mut db = open_test_db();
    let mut tx = db.transaction().expect("Failed to open transaction");

    let replica = example_replica_1();
    let ops = vec![example_add_uset_op_1()];

    // initial sync
    process_sync(&mut tx, replica, &ops).expect("Failed to sync replica with initial op");

    // try to process same op again, fail
    let res = process_sync(&mut tx, replica, &ops);
    assert!(res.is_err(), "No error when processing same operation twice: {:?}", res.unwrap());

    let err = res.unwrap_err();
    assert!(err.to_string().contains("Failed to apply incoming operations"), "Error message was incorrect: {}", err);
    assert!(err.to_string().contains("UNIQUE constraint failed: tasks.uuid"), "Error message was incorrect: {}", err);
}

#[test]
/// Add task with one client, then sync with another client and make sure we get same op back.
fn test_server_process_sync_two_clients_1() {
    let mut db = open_test_db();
    let mut tx = db.transaction().expect("Failed to open transaction");

    let replica1 = example_replica_1();
    let replica2 = example_replica_2();
    let ops = vec![example_add_uset_op_1(),];

    // sync from first client
    let res = process_sync(&mut tx, replica1, &ops);
    assert!(res.is_ok(), "Error processing incoming operation from new replica: {}", res.unwrap_err());

    let resp_ops = res.unwrap();
    assert!(resp_ops.is_empty(), "Ops in response from empty database: {:?}", resp_ops);

    // sync from second client, get back client1's ops
    let res = process_sync(&mut tx, replica2, &[]);
    assert!(res.is_ok(), "Error processing empty incoming operations from new replica: {}", res.unwrap_err());

    let resp_ops = res.unwrap();
    assert_eq!(resp_ops.len(), 1, "incorrect number of incoming ops: {:?}", resp_ops);
    assert_eq!(resp_ops, ops);
}

#[test]
/// Add tasks with one client, then sync with another client and make sure we get all tasks upon
/// first sync.
fn test_server_process_sync_two_clients_2() {
    let mut db = open_test_db();
    let mut tx = db.transaction().expect("Failed to open transaction");

    let replica1 = example_replica_1();
    let replica2 = example_replica_2();
    let ops = vec![example_add_uset_op_1(), example_remove_uset_op_1(), example_add_uset_op_2()];

    // sync from first client
    let res = process_sync(&mut tx, replica1, &ops);
    assert!(res.is_ok(), "Error processing incoming operations from new replica: {}", res.unwrap_err());

    let resp_ops = res.unwrap();
    assert!(resp_ops.is_empty(), "Ops in response from empty database: {:?}", resp_ops);

    // sync from second client, get back tasks
    let res = process_sync(&mut tx, replica2, &[]);
    assert!(res.is_ok(), "Error processing empty incoming operations from new replica: {}", res.unwrap_err());

    let resp_ops = res.unwrap();
    // only task2 because the first two ops were "add task1, remove task 1"
    let expected_ops = vec![example_add_uset_op_2(),];
    assert_eq!(resp_ops, expected_ops);
}

#[test]
/// Add tasks with one client, then sync with another client and make sure we get those ops upon
/// first sync and they are still there upon second sync, because we did not clear them.
fn test_server_process_sync_two_clients_3() {
    let mut db = open_test_db();
    let mut tx = db.transaction().expect("Failed to open transaction");

    let replica1 = example_replica_1();
    let replica2 = example_replica_2();
    let ops = vec![example_add_uset_op_1(), example_remove_uset_op_1(), example_add_uset_op_2()];

    // sync from first client
    let res = process_sync(&mut tx, replica1, &ops);
    assert!(res.is_ok(), "Error processing incoming operations from new replica: {}", res.unwrap_err());

    let resp_ops = res.unwrap();
    assert!(resp_ops.is_empty(), "Ops in response from empty database: {:?}", resp_ops);

    let expected_ops = vec![example_add_uset_op_2(),];
    // sync from second client, get back client1's ops
    let res = process_sync(&mut tx, replica2, &[]);
    assert!(res.is_ok(), "Error processing empty incoming operations from new replica: {}", res.unwrap_err());

    let resp_ops = res.unwrap();
    assert_eq!(resp_ops, expected_ops);

    // sync again and check the same ops are returned again
    let res = process_sync(&mut tx, replica2, &[]);
    assert!(res.is_ok(), "Error processing empty incoming operations from new replica: {}", res.unwrap_err());

    let resp_ops = res.unwrap();
    assert_eq!(resp_ops, expected_ops);
}

#[test]
/// Add tasks manually, sync new client, do not clear, sync again and check we still get back all
/// tasks.
fn test_server_process_sync_new_client_no_clear() {
    let mut db = open_test_db();
    let mut tx = db.transaction().expect("Failed to open transaction");

    let tasks = example_task_list();
    for task in &tasks {
        tx.add_task(&task).expect("Failed to add task");
    }

    let replica = example_replica_1();
    let res = process_sync(&mut tx, replica, &[]);
    assert!(res.is_ok(), "Error processing empty incoming operations from new replica: {}", res.unwrap_err());

    // check we get back the pre-existing db tasks as uset operations
    let resp_ops = res.unwrap();
    let expected_ops: Vec<USetOp> = tasks.iter().cloned().map(|t| USetOp::Add(t)).collect();
    assert_eq!(resp_ops, expected_ops);

    // sync again and check that we get them again, since we did not clear
    let res = process_sync(&mut tx, replica, &[]);
    assert!(res.is_ok(), "Error processing empty incoming operations from new replica: {}", res.unwrap_err());

    let resp_ops = res.unwrap();
    let expected_ops: Vec<USetOp> = tasks.into_iter().map(|t| USetOp::Add(t)).collect();
    assert_eq!(resp_ops, expected_ops);
}

fn both<T: Clone>(v1: &[T], v2: &[T]) -> Vec<T> {
    [v1,v2].concat()
}

proptest! {
    #[test]
    /// Add tasks via client 1, add tasks via sync client 2, add tasks via sync client 3, check that
    /// responses contain expected tasks.
    fn test_server_process_sync_three_clients_arb(
            ops1 in uset_add_list_arb(), ops2 in uset_add_list_arb(), ops3 in uset_add_list_arb()) {

        let mut db = open_test_db();
        let mut tx = db.transaction().expect("Failed to open transaction");

        // add all replicas to replica set before hand to avoid ordering problems with pre-existing
        // tasks on initial sync.
        let replica1 = example_replica_1();
        let replica2 = example_replica_2();
        let replica3 = example_replica_3();
        process_sync(&mut tx, replica1, &[]).unwrap();
        process_sync(&mut tx, replica2, &[]).unwrap();
        process_sync(&mut tx, replica3, &[]).unwrap();

        // sync client 1, get nothing back
        let res = process_sync(&mut tx, replica1, &ops1);
        prop_assert!(res.is_ok(), "Error processing incoming operations from new replica 1: {}", res.unwrap_err());

        let resp_ops1 = res.unwrap();
        prop_assert!(resp_ops1.is_empty());


        // sync client 2, get ops1 back
        let res = process_sync(&mut tx, replica2, &ops2);
        prop_assert!(res.is_ok(), "Error processing incoming operations from new replica 2: {}", res.unwrap_err());

        let resp_ops2 = res.unwrap();
        let expected_ops2 = ops1.clone();
        prop_assert_eq!(resp_ops2, expected_ops2, "line {}", line!());


        // sync client 3, get ops1+ops2 back
        let res = process_sync(&mut tx, replica3, &ops3);
        prop_assert!(res.is_ok(), "Error processing incoming operations from new replica 3: {}", res.unwrap_err());

        let resp_ops3 = res.unwrap();
        let expected_ops3 = both(&ops1, &ops2);
        prop_assert_eq!(resp_ops3, expected_ops3, "line {}", line!());


        // sync again with no ops for each client and make sure everything is correct: we get the
        // ops added by the other replicas

        // sync client1 again, get ops2+ops3 back
        let res = process_sync(&mut tx, replica1, &[]);
        prop_assert!(res.is_ok(), "Error processing incoming operations from second sync replica 1: {}", res.unwrap_err());

        let resp_ops1_again = res.unwrap();
        let expected_ops1_again = both(&ops2, &ops3);
        prop_assert_eq!(resp_ops1_again, expected_ops1_again, "line {}", line!());

        // sync client2 again, get ops1+ops3 back
        let res = process_sync(&mut tx, replica2, &[]);
        prop_assert!(res.is_ok(), "Error processing incoming operations from second sync replica 2: {}", res.unwrap_err());

        let resp_ops2_again = res.unwrap();
        let expected_ops2_again = both(&ops1, &ops3);
        prop_assert_eq!(resp_ops2_again, expected_ops2_again, "line {}", line!());

        // sync client3 again, get ops1+ops2 back again
        let res = process_sync(&mut tx, replica3, &[]);
        prop_assert!(res.is_ok(), "Error processing incoming operations from second sync replica 3: {}", res.unwrap_err());

        let resp_ops3_again = res.unwrap();
        let expected_ops3_again = both(&ops1, &ops2);
        prop_assert_eq!(resp_ops3_again, expected_ops3_again, "line {}", line!());
    }
}

proptest! {
    #[test]
    /// Make two databases, and process sync over a list of message two ways: all at once and one
    /// at a time. Check that they give the same result when syncing with a new client.
    fn test_server_process_sync_one_client_add_two_ways_arb(ops in uset_add_list_arb()) {
        let mut db1 = open_test_db();
        let mut db2 = open_test_db();

        let mut tx1 = db1.transaction().unwrap();
        let mut tx2 = db2.transaction().unwrap();

        let replica1 = example_replica_1();

        // sync replica 1 with database 1 one operation at a time
        for op in &ops {
            let res = process_sync(&mut tx1, replica1, &[op.clone(),]);
            prop_assert!(res.is_ok(), "Error processing single op: {}", res.unwrap_err());
            let resp_ops = res.unwrap();
            prop_assert!(resp_ops.is_empty(), "Recieved uset ops in response from database with 1 replica: {:?}", resp_ops);
        }

        // sync replica 1 with database 2 all operations at once
        let res = process_sync(&mut tx2, replica1, &ops);
        prop_assert!(res.is_ok(), "Error processing all ops at once: {}", res.unwrap_err());

        let resp_ops = res.unwrap();
        prop_assert!(resp_ops.is_empty(), "Recieved response operations after syncing database with 1 replica: {:?}", resp_ops);

        // sync replica 2 with database 1 and 2 and check responses are the same
        let replica2 = example_replica_2();

        let res1 = process_sync(&mut tx1, replica2, &[]);
        let res2 = process_sync(&mut tx2, replica2, &[]);

        prop_assert!(res1.is_ok(), "Error syncing replica 2 with database 1: {}", res1.unwrap_err());
        prop_assert!(res2.is_ok(), "Error syncing replica 2 with database 2: {}", res2.unwrap_err());

        let resp_ops1 = res1.unwrap();
        let resp_ops2 = res2.unwrap();

        prop_assert_eq!(resp_ops1, resp_ops2);

        // check that items in databases are the same directly as well
        let db1_tasks = tx1.fetch_all_tasks().expect("Failed to fetch tasks from db1");
        let db2_tasks = tx2.fetch_all_tasks().expect("Failed to fetch tasks from db2");

        prop_assert_eq!(db1_tasks, db2_tasks);
    }
}

// TODO: stateful testing where adds and removes are chosen one at a time such that causal order is
// preserved (i.e. no remove before corresponding add, but otherwise arbitrary ordering) and then
// check at each step (or just at the end) that a Vec model produces the same result


proptest! {
    #[test]
    /// Make a database, and process_sync over a list of adds and then a list of removes in two
    /// ways: one at a time and all at once, and check that both databases are empty upon sync with
    /// a new replica.
    fn test_server_process_sync_one_client_add_remove_two_ways_arb(ops in uset_add_list_arb()) {
        let mut db1 = open_test_db();
        let mut db2 = open_test_db();

        let mut tx1 = db1.transaction().unwrap();
        let mut tx2 = db2.transaction().unwrap();

        let replica1 = example_replica_1();
        let replica2 = example_replica_2();

        // register replica2 with both dbs so it accumulates all messages
        let res1 = process_sync(&mut tx1, replica2, &[]);
        let res2 = process_sync(&mut tx2, replica2, &[]);
        prop_assert!(res1.is_ok(), "Error registering replica 2 with db 1: {}", res1.unwrap_err());
        prop_assert!(res2.is_ok(), "Error registering replica 2 with db 2: {}", res2.unwrap_err());

        // first apply add operations
        let add_ops = ops.clone();

        // sync replica 1 with database 1 one operation at a time
        for op in &add_ops {
            let res = process_sync(&mut tx1, replica1, &[op.clone(),]);
            prop_assert!(res.is_ok(), "Error processing single add op: {}", res.unwrap_err());
            let resp_ops = res.unwrap();
            prop_assert!(resp_ops.is_empty(), "Recieved uset ops in response from database with 1 replica: {:?}", resp_ops);
        }

        // sync replica 1 with database 2 all operations at once
        let res = process_sync(&mut tx2, replica1, &add_ops);
        prop_assert!(res.is_ok(), "Error processing all add ops at once: {}", res.unwrap_err());

        let resp_ops = res.unwrap();
        prop_assert!(resp_ops.is_empty(), "Recieved response operations after syncing database with 1 replica: {:?}", resp_ops);

        // then turn add into remove operations and apply both ways

        let remove_ops: Vec<_> = add_ops.into_iter().map(|op| op.into_remove()).collect();
        
        // sync replica 1 with database 1 one operation at a time
        for op in &remove_ops {
            let res = process_sync(&mut tx1, replica1, &[op.clone(),]);
            prop_assert!(res.is_ok(), "Error processing single remove op: {}", res.unwrap_err());
            let resp_ops = res.unwrap();
            prop_assert!(resp_ops.is_empty(), "Recieved uset ops in response from database with 1 replica: {:?}", resp_ops);
        }

        // sync replica 1 with database 2 all operations at once
        let res = process_sync(&mut tx2, replica1, &remove_ops);
        prop_assert!(res.is_ok(), "Error processing all remove ops at once: {}", res.unwrap_err());

        let resp_ops = res.unwrap();
        prop_assert!(resp_ops.is_empty(), "Recieved response operations after syncing database with 1 replica: {:?}", resp_ops);


        // check that replica 2 has all ops from replica 1 in both dbs
        let res1 = process_sync(&mut tx1, replica2, &[]);
        prop_assert!(res1.is_ok(), "Error syncing replica 2 to retrieve replica 1's operations from db1: {}", res1.unwrap_err());
        let res2 = process_sync(&mut tx2, replica2, &[]);
        prop_assert!(res2.is_ok(), "Error syncing replica 2 to retrieve replica 1's operations from db2: {}", res2.unwrap_err());

        // check that both dbs give the same messages, that the number of messages are correct, and
        // that they are in the correct order.
        let ops1 = res1.unwrap();
        let ops2 = res2.unwrap();
        prop_assert_eq!(&ops1, &ops2);
        // each add has a corresponding remove, so 2*num ops we started with
        prop_assert!(ops1.len() == 2*ops.len());

        // first half should be original ops, second half should be remove ops
        prop_assert_eq!(&ops1[..ops.len()], ops.as_slice());
        prop_assert_eq!(&ops1[ops.len()..], remove_ops.as_slice());

        // check db is empty and new clients don't get messages

        // sync new replica 3 with database 1 and 2 and check responses are the same: db is empty,
        // so no messages
        let replica3 = example_replica_3();

        let res1 = process_sync(&mut tx1, replica3, &[]);
        let res2 = process_sync(&mut tx2, replica3, &[]);

        prop_assert!(res1.is_ok(), "Error syncing replica 3 with database 1: {}", res1.unwrap_err());
        prop_assert!(res2.is_ok(), "Error syncing replica 3 with database 2: {}", res2.unwrap_err());

        let resp_ops1 = res1.unwrap();
        let resp_ops2 = res2.unwrap();

        prop_assert!(resp_ops1.is_empty());
        prop_assert!(resp_ops2.is_empty());

        // check dbs are empty directly as well
        let db1_tasks = tx1.fetch_all_tasks().expect("Failed to fetch tasks from db1");
        let db2_tasks = tx2.fetch_all_tasks().expect("Failed to fetch tasks from db2");

        prop_assert!(db1_tasks.is_empty());
        prop_assert!(db2_tasks.is_empty());

    }
}
