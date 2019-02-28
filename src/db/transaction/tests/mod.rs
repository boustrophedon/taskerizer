mod add;
mod list;
mod set_current;
mod fetch_current;
mod remove_task;
mod pop_current;
mod remove_by_uuid;
mod store_uset_op;
mod fetch_uset_op;
mod clear_uset_op;
mod store_replica;
mod store_replica_server;
mod fetch_replica_server;

use proptest::prelude::*;
use uuid::Uuid;
use crate::sync::ReplicaUuid;

prop_compose! {
    fn arb_replica_ids()(size in 0..50usize)
        (uuids in prop::collection::vec(any::<u128>(), size))
        -> Vec<ReplicaUuid> {
            uuids.into_iter().map(|id| Uuid::from(id)).collect()
    }
}

prop_compose! {
    fn arb_server_data()(size in 0..50usize)
        (uuids in arb_replica_ids(),
        urls in prop::collection::hash_set("[a-zA-Z0-9-]+", size))
        -> Vec<(String, ReplicaUuid)> {
            urls.into_iter().zip(uuids.into_iter()).collect()
    }
}
