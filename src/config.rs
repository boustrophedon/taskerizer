use std::fs::PathBuf;

pub struct Config {
    pub db_path: PathBuf,
}

// TODO write actual implementation that creates directory and config file if not exist
// test by setting home env var to tempdir and checking that files are created
/*
impl Config {
    pub fn config() -> Config {
        Config {
            db_path: PathBuf::from("/tmp/tkzr"),
        }
    }
}
*/
