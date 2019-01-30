use proptest::prelude::*;
use uuid::Uuid;

use crate::task::Task;
use crate::sync::USetOp;

use crate::task::test_utils::{arb_task, arb_task_list};

prop_compose! {
    fn uset_add_arb()(task in arb_task()) -> USetOp {
        USetOp::Add(task)
    }
}

prop_compose! {
    fn uset_remove_arb()(uuid in any::<u128>()) -> USetOp {
        USetOp::Remove(Uuid::from_u128(uuid))
    }
}

prop_compose! {
    fn uset_add_list_arb()(tasks in arb_task_list()) -> Vec<USetOp> {
        tasks.into_iter().map(USetOp::Add).collect()
    }
}


impl USetOp {
    /// Unwrap the `Task` from an add operation. Panics if `self` is a remove operation.
    fn unwrap_add(self) -> Task {
        match self {
            USetOp::Add(task) => task,
            USetOp::Remove(uuid) => panic!("unwrap_add called on Remove: uuid {}", uuid),
        }
    }

    /// Unwrap the `Uuid` from a remove operation. Panics if `self` is an add operation.
    fn unwrap_remove(self) -> Uuid {
        match self {
            USetOp::Add(task) => panic!("unwrap_remove called on Add: task {:?}", task),
            USetOp::Remove(uuid) => uuid,
        }
    }


    /// Turn an add operation into a remove operation. Panics if `self` is already a remove
    /// operation.
    fn into_remove(self) -> USetOp {
        match self {
            USetOp::Add(task) => USetOp::Remove(task.uuid().clone()),
            USetOp::Remove(_) => panic!("turn remove into remove is not allowed"),
        }
    }
}

mod add;
mod remove;
