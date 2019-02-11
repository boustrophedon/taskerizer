use proptest::prelude::*;
use uuid::Uuid;

use crate::task::Task;
use crate::sync::{USetOp, ClientUuid};

use crate::task::test_utils::{example_task_1, example_task_2, arb_task, arb_task_list};

pub fn example_client_1() -> ClientUuid {
    Uuid::from_bytes([0,0,0,0, 0,0,0,0, 0,0,0,0, 1,2,3,4])
}
pub fn example_client_2() -> ClientUuid {
    Uuid::from_bytes([0,0,0,0, 0,0,0,0, 0,0,0,0, 1,2,3,5])
}
pub fn example_client_3() -> ClientUuid {
    Uuid::from_bytes([0,0,0,0, 0,0,0,0, 0,0,0,0, 1,2,3,6])
}

pub fn example_add_uset_op_1() -> USetOp {
    USetOp::Add(example_task_1())
}

pub fn example_add_uset_op_2() -> USetOp {
    USetOp::Add(example_task_2())
}

pub fn example_remove_uset_op_1() -> USetOp {
    USetOp::Remove(example_task_1().uuid().clone())
}

pub fn example_remove_uset_op_2() -> USetOp {
    USetOp::Remove(example_task_2().uuid().clone())
}


prop_compose! {
    pub fn uset_add_arb()(task in arb_task()) -> USetOp {
        USetOp::Add(task)
    }
}

prop_compose! {
    pub fn uset_remove_arb()(uuid in any::<u128>()) -> USetOp {
        USetOp::Remove(Uuid::from_u128(uuid))
    }
}

prop_compose! {
    pub fn uset_add_list_arb()(tasks in arb_task_list()) -> Vec<USetOp> {
        tasks.into_iter().map(USetOp::Add).collect()
    }
}

impl USetOp {
    /// Unwrap the `Task` from an add operation. Panics if `self` is a remove operation.
    pub(crate) fn unwrap_add(self) -> Task {
        match self {
            USetOp::Add(task) => task,
            USetOp::Remove(uuid) => panic!("unwrap_add called on Remove: uuid {}", uuid),
        }
    }

    /// Unwrap the `Uuid` from a remove operation. Panics if `self` is an add operation.
    pub(crate) fn unwrap_remove(self) -> Uuid {
        match self {
            USetOp::Add(task) => panic!("unwrap_remove called on Add: task {:?}", task),
            USetOp::Remove(uuid) => uuid,
        }
    }


    /// Turn an add operation into a remove operation. Panics if `self` is already a remove
    /// operation.
    pub(crate) fn into_remove(self) -> USetOp {
        match self {
            USetOp::Add(task) => USetOp::Remove(task.uuid().clone()),
            USetOp::Remove(_) => panic!("turn remove into remove is not allowed"),
        }
    }
}

