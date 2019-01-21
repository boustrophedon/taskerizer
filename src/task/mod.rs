use failure::Error;

use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Break,
    Task,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Task representation.
pub struct Task {
    task: String,
    priority: u32,
    // can't call it "break" because it's a keyword
    reward: bool,
    uuid: Uuid,
}

impl Task {
    /// Create a Task from its parts, returning an error if the parts are invalid. The task
    /// description string may not be empty or contain null bytes. The priority may not be 0.
    ///
    /// This function is for creating new Tasks from user input: it generates the Uuid internally.
    /// If you have an existing Task from a database or the network, use `Task::from_parts` and
    /// pass in the Uuid.
    pub fn new_from_parts(task: String, priority: u32, reward: bool) -> Result<Task, Error> {
        Task::from_parts(task, priority, reward, Uuid::new_v4())
    }

    /// Create a Task from its parts, returning an error if the parts are invalid. The task
    /// description string may not be empty or contain null bytes. The priority may not be 0.
    ///
    /// This function is for loading existing Tasks from disk or the network. If you want to create
    /// a new task (e.g. from user input, or for testing), use `Task::new_from_parts`.
    pub fn from_parts(task: String, priority: u32, reward: bool, uuid: Uuid) -> Result<Task, Error> {
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
            task,
            priority,
            reward,
            uuid
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

    /// Returns the Uuid of the Task.
    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    /// Is the task categorized as a task or a break
    pub fn is_break(&self) -> bool {
        self.reward
    }

    pub fn category_str(&self) -> &str {
        match self.reward {
            true => "Break",
            false => "Task",
        }
    }

    /// Format a `Task` into a single-line string
    pub fn format_row(&self, priority_column_width: usize) -> String {
        format!("{:>width$} \t {}", self.priority, self.task,
                width = priority_column_width,
        ).to_string()
    }

    /// Format a `Task` into a multiple-line string
    pub fn format_long(&self) -> String {
        format!("Task: {}\n\
            Priority: {}\n\
            Category: {}",
            self.task, self.priority, self.category_str())
            .to_string()
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) mod test_utils;
