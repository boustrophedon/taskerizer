use failure::Error;
use config::Config;

use db::{make_sqlite_backend, DBBackend};
use task::Task;

use super::Subcommand;

#[derive(StructOpt, Debug)]
pub struct Add {
    #[structopt(long = "break", short = "b")]
    /// Put this task in the "break" category
    pub reward: bool,
    /// The priority/weight used to randomly select the task
    pub priority: u32,
    /// The task description
    pub task: Vec<String>,
}

impl Subcommand for Add {
    fn run(&self, config: &Config) -> Result<Vec<String>, Error> {
        // TODO it would be nice if these "secondary validators" could be added to the structopt
        // easily. the priority 0 one probably could, but the vec string one might be difficult.
        // if i do, then the cmd tests for them can be removed and replaced with smaller ones that
        // just test those individual validators
        if self.task.len() == 0 {
            return Err(format_err!("Task cannot be empty."));
        }
        if self.priority == 0 {
            return Err(format_err!("Task cannot have priority 0 since it will never be selected."));
        }

        let task_text = self.task.join(" ");
        let task = Task {
            task: task_text.clone(),
            reward: self.reward,
            priority: self.priority,
        };

        let db = make_sqlite_backend(&config.db_path)
            .map_err(|e| format_err!("Could not acquire database connection. {}", e))?;
       
        db.add_task(&task)
            .map_err(|e| format_err!("Could not add task to database. {}", e))?;

        Ok(vec![
           format!("Task \"{}\" added to task list.", task_text),
        ])
    }
}
