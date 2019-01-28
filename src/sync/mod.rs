use uuid::Uuid;
use crate::task::Task;

type ClientUuid = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
enum USetOp {
    Add (Task),
    Remove (Uuid),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct USetOpMsg {
    op: USetOp,
    deliver_to: ClientUuid,
}

