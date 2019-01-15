use failure::Error;

use crate::db::DBBackend;
use crate::commands::Subcommand;
use crate::selection::WeightedRandom;

#[derive(Debug)]
pub struct Complete;

impl Subcommand for Complete {
    fn run(&self, tx: &impl DBBackend) -> Result<Vec<String>, Error> {
        let mut selector = WeightedRandom::new(0.5); // FIXME: this will be removed when we remove the selector from complete_current_task
        let res = tx.complete_current_task(&mut selector)
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
