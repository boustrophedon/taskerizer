use std::path::PathBuf;

use failure::Error;

use db::{DBBackend, make_sqlite_backend};

/// Configuration parameters.
pub struct Config {
    /// Location of the database directory.
    pub db_path: PathBuf,
    /// The probability of choosing a break when choosing a new task.
    pub break_cutoff: f32,
}

impl Config {
    /// Get a connection to the database at the location specified by the config file.
    pub fn db<'a>(&'a self) -> Result<impl DBBackend + 'a, Error> {
        return make_sqlite_backend(&self.db_path)
            .map_err(|e| format_err!("Could not acquire database connection. {}", e));
    }


    // TODO write actual implementation that creates directory and config file if not exist
    // test by setting home env var to tempdir and checking that files are created
    /*
    pub fn config() -> Config {
        Config {
            db_path: PathBuf::from("/tmp/tkzr"),
        }
    }
    */
}
