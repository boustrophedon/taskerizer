use failure::Error;

use crate::db::DBBackend;
use crate::commands::Subcommand;

#[derive(Debug)]
pub struct Complete;

impl Subcommand for Complete {
    fn run(&self, tx: &impl DBBackend) -> Result<Vec<String>, Error> {
        let res = tx.complete_current_task()
            .map_err(|e| format_err!("Could not complete current task. {}", e))?;
       
        if let Some(completed_current) = res {
            return Ok(vec![
                      format!("Task \"{}\" completed.\n", completed_current.task()),
            ]);
        }
        else {
            return Ok(vec!["No tasks.".to_string()]);
        }
    }
}
