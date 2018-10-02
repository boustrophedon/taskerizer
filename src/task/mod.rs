use failure::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Task representation. Priority must not be 0.
pub struct Task {
    pub task: String,
    pub priority: u32,
    pub reward: bool,
}

impl Task {
    pub fn from_parts(task: String, priority: u32, reward: bool) -> Result<Task, Error> {
        if task.len() == 0 {
            return Err(format_err!("Empty task description not allowed when creating task from parts."));
        }
        if priority == 0 {
            return Err(format_err!("Zero priority not allowed when creating task from parts."));
        }

        Ok(Task {
            task: task,
            priority: priority,
            reward: reward
        })
    }
}

#[cfg(test)]
mod test {
    use super::Task;
    use proptest::arbitrary::any;

    #[test]
    fn test_task_nonempty_task() {
        let res = Task::from_parts("".to_string(), 1, false);
        assert!(res.is_err(), "Task was valid with empty task description");
        let err = res.unwrap_err();
        assert!(err.to_string().contains("Empty task description"), "Incorrect error with empty task description: {}", err);
    }

    #[test]
    fn test_task_zero_priority() {
        let res = Task::from_parts("a".to_string(), 0, false);
        assert!(res.is_err(), "Task was valid with zero priority");
        let err = res.unwrap_err();
        assert!(err.to_string().contains("Zero priority"), "Incorrect error with zero priority: {}", err);
    }

    proptest! {
        #[test]
        /// Create a valid task and check that Task::from_parts works. Valid task has at least one
        /// character and priority at least 1.
        fn test_task_valid_arb_task(task in ".+", priority in 1u32.., reward in any::<bool>()) {
            let res = Task::from_parts(task, priority, reward);
            assert!(res.is_ok(), "Task made from good parts returned an error: {}", res.unwrap_err());
        }
    }
}

#[cfg(test)]
pub(crate) mod test_utils;
