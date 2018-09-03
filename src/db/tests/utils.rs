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

pub fn example_task_break() -> Task {
    Task {
        task: "another tesk task with break set".to_string(),
        priority: 1,
        reward: true,
    }
}
