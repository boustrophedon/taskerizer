use failure::Error;

use crate::db::DBBackend;
use crate::selection::SelectionStrategy;

#[derive(Debug)]
pub struct Skip;

impl Skip {
    pub fn run(&self, tx: &impl DBBackend, selector: &mut dyn SelectionStrategy) -> Result<Vec<String>, Error> {
        let original_task_opt = tx.fetch_current_task()
            .map_err(|e| format_err!("Could not fetch current task. {}", e))?;

        tx.skip_current_task(selector)
            .map_err(|e| format_err!("Could not skip current task. {}", e))?;
       
        let opt = tx.fetch_current_task()
            .map_err(|e| format_err!("Could not fetch current task. {}", e))?;

        if let Some(current_task) = opt {
            return Ok(vec![
                      format!("Current task is now \"{}\".\n", current_task.task()),
            ]);
        }
        else if let Some(original_task) = original_task_opt {
            // if there was a task originally and after skipping there is none, then the
            // original one was the only one in the db, so set it back.
            // we don't actually set it here, but it will be selected in TKZCmd::run.

            return Ok(vec![
                      format!("Current task is now \"{}\".\n", original_task.task()),
            ]);
        }
        else {
            return Ok(vec!["No tasks.".to_string()]);
        }
    }
}
