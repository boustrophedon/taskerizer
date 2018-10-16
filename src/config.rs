use std::path::PathBuf;

use failure::Error;

use db::{DBBackend, make_sqlite_backend};

pub struct Config {
    pub db_path: PathBuf,
}
impl Config {
    /// Get a connection to the database at the location specified by the config file.
    pub fn db(&self) -> Result<impl DBBackend, Error> {
        // we clone here because the `impl DBBackend` return causes a lifetime error because I
        // guess the compiler can't tell that we're actually using SqliteBackend, which doesn't
        // keep a ref to the db_path. it's weird that it doesn't have that problem if i were to
        // inline this in TKZCmd::dispatch though, so I feel like I could maybe add a lifetime
        // somehow in the return type?
        return make_sqlite_backend(self.db_path.clone())
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
