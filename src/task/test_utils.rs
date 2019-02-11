use proptest::prelude::*;

use uuid::Uuid;

use crate::task::Task;

// we can't currently make these statics (without using lazy_static)
// it should be possible on nightly due to compile time evaluation

pub fn example_task_1() -> Task {
    Task {
        task: "test task please ignore".to_string(),
        priority: 1,
        reward: false,
        uuid: Uuid::from_bytes([0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,1]),
    }
}
pub fn example_task_1_dup() -> Task {
    Task {
        task: "test task please ignore".to_string(),
        priority: 1,
        reward: false,
        uuid: Uuid::from_bytes([0,0,0,0, 0,0,0,0, 0,0,0,1, 0,0,0,1]),
    }
}


pub fn example_task_2() -> Task {
    Task {
        task: "test task please ignore 2".to_string(),
        priority: 12,
        reward: false,
        uuid: Uuid::from_bytes([0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,2]),
    }
}

pub fn example_task_3() -> Task {
    Task {
        task: "just another task".to_string(),
        priority: 2,
        reward: false,
        uuid: Uuid::from_bytes([0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,3]),
    }
}

pub fn example_task_break_1() -> Task {
    Task {
        task: "another tesk task with break set".to_string(),
        priority: 1,
        reward: true,
        uuid: Uuid::from_bytes([0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,1,0]),
    }
}

pub fn example_task_break_2() -> Task {
    Task {
        task: "break with high priority".to_string(),
        priority: 99,
        reward: true,
        uuid: Uuid::from_bytes([0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,2,0]),
    }
}

pub fn example_task_list() -> Vec<Task> {
    vec![
        example_task_1(),
        example_task_break_1(),
        example_task_2(),
        example_task_break_2(),
    ]
}

impl Task {
    pub fn example_invalid_empty_desc() -> Task {
        Task {
            task: "".to_string(),
            priority: 1,
            reward: false,
            uuid: Uuid::from_bytes([255,255,255,1, 0,0,0,0, 0,0,0,0, 0,0,0,0]),
        }
    }

    pub fn example_invalid_zero_priority() -> Task {
        Task {
            task: "test task".to_string(),
            priority: 0,
            reward: true,
            uuid: Uuid::from_bytes([255,255,255,2, 0,0,0,0, 0,0,0,0, 0,0,0,0]),
        }
    }
}


// proptest gen functions

// TODO: do i need to generate the bytes for the uuids here via proptest and pass them in? not sure
// that it's necessary, but if there's a bug the example shrinking might have a hard time because
// the uuids will be different when it tries to shrink.

prop_compose! {
    pub fn arb_task()(task in "[^\x00]+",
                  priority in 1u32..,
                  reward in any::<bool>()) -> Task {
        Task::new_from_parts(task, priority, reward).expect("invalid parts")
    }
}

prop_compose! {
    pub fn arb_task_bounded()(task in "[^\x00]{1,50}",
                  priority in 1..100u32,
                  reward in any::<bool>()) -> Task {
        Task::new_from_parts(task, priority, reward).expect("invalid parts")
    }
}

prop_compose! {
    pub fn arb_task_list()(tasks in prop::collection::vec(arb_task(), 1..100))
        -> Vec<Task> {
            tasks
    }
}


prop_compose! {
    pub fn arb_task_list_bounded()(tasks in prop::collection::vec(arb_task_bounded(), 1..100))
        -> Vec<Task> {
            tasks
    }
}

