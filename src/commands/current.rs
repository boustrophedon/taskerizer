use failure::Error;

use crate::db::DBBackend;

use super::Subcommand;

#[derive(StructOpt, Debug)]
pub struct Current {
    #[structopt(long = "top")]
    /// Displays the task with the highest priority instead of the currently selected task
    pub top: bool,
}

impl Subcommand for Current {
    fn run(&self, tx: &impl DBBackend) -> Result<Vec<String>, Error> {
        let res = tx.fetch_current_task()
            .map_err(|e| format_err!("Could not get current task from database. {}", e))?;
       
        if let Some(current) = res {
            return Ok(vec![
                      format!("{}\n", current.task()),
                      format!("Category: {}", current.category_str()),
                      format!("Priority: {}", current.priority().to_string()),
            ]);
        }
        else {
            return Ok(vec!["No tasks.".to_string()]);
        }
    }
}
