use failure::Error;
use config::Config;

use super::Subcommand;

#[derive(Debug)]
pub struct List;

impl Subcommand for List {
    fn run(&self, config: &Config) -> Result<Vec<String>, Error> {
        Ok(vec!["Item\tTask\tPriority".to_string(),
        "1\thello this is a test\t1".to_string(),])
    }
}
