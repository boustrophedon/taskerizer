use failure::{Error, err_msg};
use config::Config;

use structopt::StructOpt;

/// Default command when none is given: display the current selected task.
const DEFAULT_COMMAND: TKZCmd = TKZCmd::Current(Current{top: false});

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
        Ok(vec!["Task \"hello this is a test\" added to task list.".to_string()])
    }
}

// --- subcommand parameter structs

#[derive(StructOpt, Debug)]
pub struct Add {
    #[structopt(long = "break", short = "b")]
    /// Put this task in the "break" category
    pub reward: bool,
    /// The priority/weight used to randomly select the task
    pub priority: u32,
    /// The task description
    pub task: Vec<String>,
}


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
pub struct Current {
    #[structopt(long = "top")]
    /// Displays the task with the highest priority instead of the currently selected task
    pub top: bool,
}


#[derive(StructOpt, Debug)]
pub struct Skip {
    #[structopt(long = "any")]
    /// Chooses a task from any category instead of the same as the current task's
    pub any: bool,
}
