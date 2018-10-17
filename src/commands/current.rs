use failure::Error;

use db::DBBackend;
use task::Task;

use super::Subcommand;

#[derive(StructOpt, Debug)]
pub struct Current {
    #[structopt(long = "top")]
    /// Displays the task with the highest priority instead of the currently selected task
    pub top: bool,
}

fn format_category(task: &Task) -> String {
    match task.is_break() {
        true => "Break",
        false => "Task",
    }.to_string()
}

impl Subcommand for Current {
    fn run(&self, db: &mut impl DBBackend) -> Result<Vec<String>, Error> {
        let res = db.get_current_task()
            .map_err(|e| format_err!("Could not get current task from database. {}", e))?;
       
        if let Some(current) = res {
            return Ok(vec!["Current task:\n".to_string(),
                      format!("Category: {}", format_category(&current)),
                      format!("Priority: {}", current.priority().to_string()),
                      format!("Description: {}", current.task()),
            ]);
        }
        else {
            return Ok(vec!["No tasks.".to_string()]);
        }
    }
}
