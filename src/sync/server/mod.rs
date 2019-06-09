use failure::Error;

use crate::db::DBBackend;
use crate::sync::{ReplicaUuid, USetOp, USetOpMsg, apply_all_uset_ops};


#[cfg(test)]
mod tests;

/// Process an incoming sync operation.
///
/// If `incoming_replica` is an unknown replica, we add it to the replica set, fetch all tasks in
/// the database, turn them into `USetOp::Add` operations, store them in the database, and send
/// them as the response to the client.
///
/// If `incoming_replica` is a known replica, we fetch all its unsynced operations and send them as
/// the response to the client.
///
/// The `incoming_ops` from the replica in either case are processed after the response is
/// computed. The add and remove operations are applied to the database, and then the operations
/// are stored as unsynced `USetOpMsg`s for each known replica.
///
/// `USetOps` are sent in the order they are received, with the exception that upon initial sync,
/// the tasks currently in the database are returned as `USetOp`s in the sort order defined by
/// `DBBackend::fetch_all_tasks`, which should be "sorted by category first and then priority
/// second"
pub fn process_sync(tx: &mut impl DBBackend, incoming_replica: ReplicaUuid, incoming_ops: &[USetOp]) -> Result<Vec<USetOp>, Error> {
    // we can compute this via two ways:
    // first, check if the replica_id is known, and then if it is fetch its unsynced ops, and if
    // unknown, fetch all tasks and turn them into ops
    //
    // alternative, slightly optimized: fetch unsynced ops for replica, and if empty then check if it
    // is unknown, and if it is unknown then fetch tasks and turn into ops. this optimizes latency
    // slightly for the more common case that an existing member is syncing.

    // FIXME: this can be factored out into multiple functions

    let replicas = tx.fetch_replicas()
        .map_err(|e| format_err!("Failed to fetch replicas while processing incoming sync for client {}: {}", incoming_replica, e))?;

    let results: Vec<USetOp>;
    if replicas.iter().map(|(r, _)| r).find(|&r| r == &incoming_replica).is_some() {
        // replica_id is known
        let messages = tx.fetch_uset_op_msgs(&incoming_replica)
            .map_err(|e| format_err!("Failed to fetch uset op messages while processing incoming sync for client {}: {}",
                                     incoming_replica, e))?;

        results = messages.into_iter().map(|msg| msg.op).collect();
    }
    else {
        // replica_id is unknown, gather all tasks and add them to unsynced messages table, and
        // send them to replica.

        tx.store_replica_client(&incoming_replica)
            .map_err(|e| format_err!("Failed to store new client replica {} while processing incoming sync: {}", &incoming_replica, e))?;

        let tasks = tx.fetch_all_tasks()
            .map_err(|e| format_err!("Failed to fetch all tasks while processing incoming sync for new client {}: {}",
                                     incoming_replica, e))?;

        let ops = tasks.into_iter().map(|t| USetOp::Add(t));
        let op_msgs = ops.clone().map(|op| USetOpMsg { op, deliver_to: incoming_replica });

        for msg in op_msgs {
            tx.store_uset_op_msg(&msg)
                .map_err(|e| 
                     format_err!("Failed to store uset op message for new client {} while processing incoming sync: {}",
                                 incoming_replica, e))?;
        }

        results = ops.collect();
    }

    // Apply incoming ops after getting existing messages.
    // move this into separate function
    apply_all_uset_ops(tx, incoming_ops)
        .map_err(|e| format_err!("Failed to apply incoming operations from client {} while processing incoming sync: {}",
                                 incoming_replica, e))?;
    for (replica_id, _) in &replicas {
        if *replica_id == incoming_replica { 
            continue;
        }
        for op in incoming_ops {
            let msg = USetOpMsg { op: op.clone(), deliver_to: *replica_id };
            tx.store_uset_op_msg(&msg)
                .map_err(|e| format_err!("Failed to store uset op message for client {} while processing incoming sync from client {}: {}",
                                        incoming_replica, replica_id, e))?;
        }
    }

    return Ok(results);
}

/// Process an incoming clear operation.
///
/// This will clear all `USetOpMsg`s in the database for the given replica with UUid `incoming_replica`.
///
/// Note that we do not keep track of which operations were successfully synced - if replica A
/// syncs, and then replica B syncs before replica A clears again, any operations from B that A
/// would have recieved by syncing will be cleared. See `notes.md` for more information.
pub fn process_clear(tx: &mut impl DBBackend, incoming_replica: ReplicaUuid) -> Result<(), Error> {
    tx.clear_uset_op_msgs(&incoming_replica)
        .map_err(|e| format_err!("Error processing clear with replica {}: {}", incoming_replica, e))
}
