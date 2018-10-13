use failure::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Task representation.
pub struct Task {
    task: String,
    priority: u32,
    // can't call it "break" because it's a keyword
    reward: bool,
}

impl Task {
    /// Create a Task from its parts, returning an error if the parts are invalid. The task
    /// description string may not be empty or contain null bytes. The priority may not be 0.
    pub fn from_parts(task: String, priority: u32, reward: bool) -> Result<Task, Error> {
        if task.len() == 0 {
            return Err(format_err!("Empty task description when creating task."));
        }
        if priority == 0 {
            return Err(format_err!("Zero priority when creating task."));
        }
        if task.contains("\x00") {
            return Err(format_err!("Null bytes in task description when creating task."));
        }

        Ok(Task {
            task: task,
            priority: priority,
            reward: reward
        })
    }

    /// Returns the task description.
    pub fn task(&self) -> &str {
        &self.task
    }

    /// Returns the task priority.
    pub fn priority(&self) -> u32 {
        self.priority
    }

    /// Is the task categorized as a task or a break
    pub fn is_break(&self) -> bool {
        self.reward
    }

    /// Format a `Task` into a single-line string
    pub fn format_row(&self, priority_column_width: usize) -> String {
        format!("{:>width$} \t {}", self.priority, self.task,
                width = priority_column_width,
        ).to_string()
    }

    /*
    /// Format a `Task` into a multiple-line string
    pub fn format_long(&self) -> String {
    }
    */
}

#[cfg(test)]
mod test {
    use super::Task;
    use proptest::arbitrary::any;

    #[test]
    fn test_task_fmt_row_1() {
        let task = super::test_utils::example_task_1();
        let row = task.format_row(1);
        assert_eq!(row, "1 \t test task please ignore");

        let row = task.format_row(2);
        assert_eq!(row, " 1 \t test task please ignore");

        let row = task.format_row(4);
        assert_eq!(row, "   1 \t test task please ignore");
    }

    #[test]
    fn test_task_fmt_row_2() {
        let task = super::test_utils::example_task_2();
        let row = task.format_row(1);
        assert_eq!(row, "12 \t test task please ignore 2");

        let row = task.format_row(2);
        assert_eq!(row, "12 \t test task please ignore 2");

        let row = task.format_row(4);
        assert_eq!(row, "  12 \t test task please ignore 2");
    }

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

    #[test]
    fn test_task_null_desc() {
        let res = Task::from_parts("\x00".to_string(), 1, false);
        assert!(res.is_err(), "Task was valid with null byte in string");
        let err = res.unwrap_err();
        assert!(err.to_string().contains("Null byte"), "Incorrect error with null byte: {}", err);

        // null bytes not at beginning
        let res = Task::from_parts("test task \x00 \x00 test".to_string(), 1, false);
        assert!(res.is_err(), "Task was valid with null byte in string");
        let err = res.unwrap_err();
        assert!(err.to_string().contains("Null byte"), "Incorrect error with null byte: {}", err);

        // null byte just at end
        let res = Task::from_parts("test task \x00".to_string(), 1, false);
        assert!(res.is_err(), "Task was valid with null byte in string");
        let err = res.unwrap_err();
        assert!(err.to_string().contains("Null byte"), "Incorrect error with null byte: {}", err);
    }

    proptest! {
        #[test]
        /// Create a valid task and check that Task::from_parts works. Valid task has at least one
        /// character, no null bytes, and priority at least 1.
        fn test_task_valid_arb_task(task in "[^\x00]+", priority in 1u32.., reward in any::<bool>()) {
            let res = Task::from_parts(task, priority, reward);
            assert!(res.is_ok(), "Task made from good parts returned an error: {}", res.unwrap_err());
        }
    }
}

#[cfg(test)]
pub(crate) mod test_utils;
