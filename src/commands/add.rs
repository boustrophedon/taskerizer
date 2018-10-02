use failure::Error;

use db::DBBackend;
use task::Task;

use super::Subcommand;

// TODO I changed the parsing code to be part of structopt but now I feel like the actual code is
// more fragile, though really it was fragile the whole time because the main problem is that I
// don't have a Task::from_parts() that returns a Result and errs when the task length is 0 or the
// priority is 0. Once I do that, I can get rid of the duplicated asserts and should_panic tests.
// 
// Currently the duplication is: the is_nonempty code below, the asserts in run, and the
// #[should_panic] tests. Once I write the Task::from_parts tests and stuff, I can get rid of the
// asserts here and the should_panic tests. Then eventually I should test the parser stuff here by
// using the assert_cmd/assert_cli crate.

/// Parse a nonempty string from a command line argument
fn is_str_nonempty(arg: &str) -> Result<String, Error> {
    let s: String = arg.parse().map_err(|e| format_err!("Unable to parse {} as valid string: {}", arg, e))?;
    if s.len() == 0 {
        return Err(format_err!("Task description cannot be empty."));
    }
    return Ok(s);
}

/// Parse a nonzero u32 from a command line argument
fn is_u32_nonzero(arg: &str) -> Result<u32, Error> {
    let p: u32 = arg.parse().map_err(|e| format_err!("Unable to parse {} as valid integer: {}", arg, e))?;
    if p == 0 {
        return Err(format_err!("Task priority cannot be 0 because it would never be selected."));
    }
    return Ok(p);
}

#[derive(StructOpt, Debug)]
pub struct Add {
    #[structopt(long = "break", short = "b")]
    /// Put this task in the "break" category
    pub reward: bool,
    #[structopt(parse(try_from_str = "is_str_nonempty"))]
    /// The task description.
    pub task: String,
    #[structopt(default_value = "1", parse(try_from_str = "is_u32_nonzero"))]
    /// The priority/weight used to randomly select the task.
    pub priority: u32,
}

impl Subcommand for Add {
    fn run(&self, db: &impl DBBackend) -> Result<Vec<String>, Error> {
        assert!(self.task.len() > 0, "Task description cannot be empty.");
        assert!(self.priority > 0, "Task priority cannot be zero.");

        let task = Task {
            task: self.task.clone(),
            reward: self.reward,
            priority: self.priority,
        };

        db.add_task(&task)
            .map_err(|e| format_err!("Could not add task to database. {}", e))?;

        Ok(vec![
           format!("Task \"{}\" added to task list.", self.task),
        ])
    }
}
