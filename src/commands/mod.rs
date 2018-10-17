use failure::{Error, err_msg};

use structopt::StructOpt;

use db::DBBackend;
use config::Config;

use rand::prelude::*;
use rand::distributions::Uniform;

/// Default command when none is given: display the current selected task.
const DEFAULT_COMMAND: TKZCmd = TKZCmd::Current(Current{top: false});

// subcommand trait

trait Subcommand {
    fn run(&self, db: &mut impl DBBackend) -> Result<Vec<String>, Error>;
}

#[derive(StructOpt, Debug)]
#[structopt(name = "tkz")]
/// Taskerizer is a task randomizer. It's a todo list where each item has a weight, and `tkz
/// current` (or just `tkz`) will produce a task selected at random using the given weights. You can also add
/// categories (currently limited to a single category called "break") and the next task will be
/// selected with a separate weighted probability from the categories. This makes taskerizer a kind
/// of skinner box if you use break tasks like "watch youtube videos for 15 minutes", but it could also
/// be used for things like "take a walk for 10 minutes", or even literally just "take a break".
pub struct TKZArgs {
    /// If no command is given, defaults to current - show the current task
    #[structopt(subcommand)]
    pub cmd: Option<TKZCmd>,
}

impl TKZArgs {
    /// Convenience function so we don't have to import structopt trait into main.rs
    pub fn get_args() -> TKZArgs {
        TKZArgs::from_args()
    }

    pub fn cmd(&self) -> &TKZCmd {
        match self.cmd {
            Some(ref cmd) => &cmd,
            None => &DEFAULT_COMMAND,
        }
    }
}

// --- subcommand enum

#[derive(StructOpt, Debug)]
pub enum TKZCmd {
    #[structopt(name = "add")]
    /// Add a task to the task list.
    Add(Add),

    #[structopt(name = "break")]
    /// Skip the current task and choose a break instead, or, if the `-p` flag is present, change
    /// the probability of a break being selected.
    Break(Break),

    #[structopt(name = "list")]
    /// List all tasks.
    List,

    #[structopt(name = "current")]
    /// Display the current task.
    Current(Current),

    #[structopt(name = "complete")]
    /// Mark the current task as completed.
    Complete,

    #[structopt(name = "skip")]
    /// Skip the current task, choosing a new one at random.
    Skip(Skip),
}

impl TKZCmd {
    pub fn dispatch(&self, config: &Config) -> Result<Vec<String>, Error> {
        let mut db = config.db()?;
        let break_cutoff = config.break_cutoff;
        // get other stuff from config, etc...

        // choose random parameters for choose_current_task
        let mut rng = thread_rng();
        let task_p = Uniform::new_inclusive(0.0, 1.0).sample(&mut rng);
        let category_p = Uniform::new_inclusive(0.0, 1.0).sample(&mut rng);

        return self.run(&mut db, (task_p, category_p, break_cutoff));
    }

    fn run(&self, db: &mut impl DBBackend, choose_current_params: (f32, f32, f32)) -> Result<Vec<String>, Error> {
        let output = match self {
            TKZCmd::Add(add) => add.run(db),
            TKZCmd::List => {let l = List; l.run(db)},
            TKZCmd::Current(current) => current.run(db),
            _ => unimplemented!(),
        };

        // if there is a current task, we don't have to update, just return the output.
        let current_task = db.get_current_task()
            .map_err(|err| format_err!("Error getting current task while choosing new task after executing command: {}", err))?;
        if current_task.is_some() {
            return output;
        }
        // otherwise, choose new one

        // TODO: refactor into separate choose_selection_category

        let (task_p, category_p, break_cutoff) = choose_current_params;

        let tasks = db.get_all_tasks()
            .map_err(|err| format_err!("Failed to get tasks while choosing new task after executing command: {}", err))?;
        let num_breaks = tasks.iter().filter(|&t| t.is_break()).count();
        let num_tasks = tasks.len() - num_breaks;

        let mut choose_break = None;
        // if there are tasks of both kinds, use the passed probability to choose which category to
        // use:
        if num_breaks > 0 && num_tasks > 0 {
            choose_break = Some(category_p < break_cutoff);
        }
        // otherwise, just use whichever we have
        else if num_breaks > 0 {
            choose_break = Some(true);
        }
        else if num_tasks > 0 {
            choose_break = Some(false);
        }

        if let Some(is_break) = choose_break {
            db.choose_current_task(task_p, is_break)
                .map_err(|err| format_err!("Failed to choose new task after executing command: {}", err))?;
        }

        return output;
    }
}

#[cfg(test)]
pub mod test_utils;
#[cfg(test)]
mod test_dispatch;

// --- subcommand parameter structs

mod add;
pub use self::add::Add;

mod list;
pub use self::list::List;

mod current;
pub use self::current::Current;

#[derive(StructOpt, Debug)]
pub struct Break {
    /// The probability as a decimal to select a task from break. Must be less than 1.0.
    #[structopt(short = "p", long = "probability", parse(try_from_str = "parse_prob"))]
    pub p: Option<f32>,
}

/// Parse a float representing a probability from a string for the change-prob command
fn parse_prob(arg: &str) -> Result<f32, Error> {
    let p: f32 = arg.parse().map_err(|e| format_err!("unable to parse \"{}\": {}", arg, e))?;
    if p >= 1.0 {
        return Err(err_msg("probability must be less than 1"));
    }
    if p < 0.0 {
        return Err(err_msg("probability must be greater than 0"));
    }

    Ok(p)
}

#[derive(StructOpt, Debug)]
pub struct Skip {
    #[structopt(long = "any")]
    /// Chooses a task from any category instead of the same as the current task's
    pub any: bool,
}
