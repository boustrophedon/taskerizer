use failure::Error;
use config::Config;

use db::{make_sqlite_backend, DBBackend};
//use task::Task;

use super::Subcommand;

#[derive(Debug)]
pub struct List;

impl Subcommand for List {
    fn run(&self, config: &Config) -> Result<Vec<String>, Error> {
        let db = make_sqlite_backend(&config.db_path)
            .map_err(|e| format_err!("Could not acquire database connection. {}", e))?;

        let tasks = db.get_all_tasks()
            .map_err(|e| format_err!("Could not get tasks from database. {}", e))?;

        // TODO implement `Task::format_row`, `Task::description` instead of doing it inline here
        let mut output = vec!["Item\tTask\tPriority".to_string(),];
        output.extend(
            tasks.iter().enumerate().map(|(i, task)| {
                format!("{}\t{}\t{}", i+1, task.task, task.priority)
            })
        );

        Ok(output)
    }
}
