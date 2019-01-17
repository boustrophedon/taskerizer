use failure::{Error, err_msg};

use structopt::StructOpt;

use crate::db::DBBackend;
use crate::config::Config;
use crate::selection::WeightedRandom;
use crate::selection::SelectionStrategy;

/// Default command when none is given: display the current selected task.
const DEFAULT_COMMAND: TKZCmd = TKZCmd::Current(Current{top: false});

// subcommand trait

trait Subcommand {
    fn run(&self, tx: &impl DBBackend) -> Result<Vec<String>, Error>;
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
    /// Skip the current task and choose a break instead.
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
    /// Skip the current task and choose a new one. If there is only one task in the database, it
    /// will be chosen again.
    Skip,
}

impl TKZCmd {
    pub fn dispatch(&self, config: &Config) -> Result<Vec<String>, Error> {
        let mut db = config.db()?;
        let tx = db.transaction()?;
        let break_cutoff = config.break_cutoff;
        let mut selector = WeightedRandom::new(break_cutoff);
        // get other stuff from config, etc...

        let res = self.run(&tx, &mut selector);
        if res.is_ok() {
            tx.finish()?;
        }
        return res;
    }

    fn run(&self, tx: &impl DBBackend, selector: &mut dyn SelectionStrategy) -> Result<Vec<String>, Error> {
        let output = match self {
            TKZCmd::Add(add) => add.run(tx),
            TKZCmd::List => {let l = List; l.run(tx)},
            TKZCmd::Current(current) => current.run(tx),
            TKZCmd::Complete => {let c = Complete; c.run(tx)},
            TKZCmd::Skip => {let s = Skip; s.run(tx, selector)},
            _ => unimplemented!(),
        };

        let current_task = tx.fetch_current_task()
            .map_err(|err| format_err!("Error getting current task while choosing new task after executing command: {}", err))?;
        // if there is no current task, choose a new one.
        if current_task.is_none() {
            tx.select_current_task(selector)?;
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

mod complete;
pub use self::complete::Complete;

mod skip;
pub use self::skip::Skip;

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
