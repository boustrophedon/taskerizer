use crate::task::Task;
use crate::commands::{TKZCmd, Add};

pub fn add_from_task(task: &Task) -> TKZCmd {
    TKZCmd::Add( Add {
        reward: task.is_break(),
        task: task.task().to_string(),
        priority: task.priority(),
    })
}
