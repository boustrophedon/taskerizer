use proptest::prelude::*;

use task::Task;

// we can't currently make these statics (without using lazy_static)
// it should be possible on nightly due to compile time evaluation

pub fn example_task_1() -> Task {
    Task {
        task: "test task please ignore".to_string(),
        priority: 1,
        reward: false,
    }
}

pub fn example_task_2() -> Task {
    Task {
        task: "test task please ignore 2".to_string(),
        priority: 12,
        reward: false,
    }
}

pub fn example_task_3() -> Task {
    Task {
        task: "just another task".to_string(),
        priority: 2,
        reward: false,
    }
}

pub fn example_task_break_1() -> Task {
    Task {
        task: "another tesk task with break set".to_string(),
        priority: 1,
        reward: true,
    }
}

pub fn example_task_break_2() -> Task {
    Task {
        task: "break with high priority".to_string(),
        priority: 99,
        reward: true,
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

// proptest gen functions

prop_compose! {
    [pub] fn arb_task()(task in any::<String>(),
                  priority in 1u32..,
                  reward in any::<bool>()) -> Task {
        Task {
            task: task,
            priority: priority,
            reward: reward,
        }
    }
}

prop_compose! {
    [pub] fn arb_task_bounded()(task in ".{0,50}",
                  priority in 1..100u32,
                  reward in any::<bool>()) -> Task {
        Task {
            task: task,
            priority: priority,
            reward: reward,
        }
    }
}

prop_compose! {
    [pub] fn arb_task_list()(tasks in prop::collection::vec(arb_task(), 1..100))
        -> Vec<Task> {
            tasks
    }
}


prop_compose! {
    [pub] fn arb_task_list_bounded()(tasks in prop::collection::vec(arb_task_bounded(), 1..100))
        -> Vec<Task> {
            tasks
    }
}
