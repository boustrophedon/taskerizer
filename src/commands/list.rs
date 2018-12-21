use failure::Error;

use crate::db::DBBackend;
//use crate::task::Task;

use super::Subcommand;

#[derive(Debug)]
pub struct List;

impl Subcommand for List {
    fn run(&self, db: &mut impl DBBackend) -> Result<Vec<String>, Error> {

        let tasks = db.fetch_all_tasks()
            .map_err(|e| format_err!("Could not get tasks from database. {}", e))?;

        let mut output = vec!["Priority \t Task".to_string(),];
        output.extend(
            tasks.iter().map(|task| {
                task.format_row(4)
            })
        );

        Ok(output)
    }
}
