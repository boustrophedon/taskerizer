use failure::Error;
use config::Config;

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
        if self.task.len() == 0 {
            return Err(format_err!("Task cannot be empty."));
        }
        Ok(vec!["Task \"hello this is a test\" added to task list.".to_string()])
    }
}
