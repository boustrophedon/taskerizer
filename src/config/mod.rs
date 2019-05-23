use std::io::prelude::*;
use std::fs::File;
use std::str::FromStr;

use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use failure::Error;

use crate::db::SqliteBackend;

#[cfg(test)]
mod tests;

const DEFAULT_BREAK_CUTOFF: f32 = 0.35;

/// Configuration parameters.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    /// Location of the database directory.
    pub db_path: PathBuf,
    /// The probability of choosing a break when choosing a new task.
    pub break_cutoff: f32,
}

// creation and acquisition functions
impl Config {
    /// Opens existing or creates new configuration file, relative to base directory `path` if
    /// specified.
    ///
    /// If `user_config` is None, uses the `directories` crate to find the config directory on each
    /// platform and looks for or creates a `taskerizer/config.toml` file inside that directory.
    ///
    /// If `user_config` contains a `&Path`, look for the config file at that path. It is not
    /// created if there is no file there, and an error is returned.
    ///
    /// The default value of the db_path upon creation of the config file is the local project data
    /// directory given by the `directories` crate.
    pub fn new_in(user_config: Option<PathBuf>) -> Result<Config, Error> {
        let config_filename = match user_config {
            Some(config_filename) => {
                if !config_filename.is_file() {
                    return Err(format_err!("Config file path was given but there was no config file there: {}", 
                                           config_filename.display()));
                }
                else {
                    config_filename
                }
            }
            None => {
                let config_filename = Config::default_config_filename();
                if !config_filename.is_file() {
                    return Config::write_default_config();
                }
                else {
                    config_filename
                }
            }
        };

        let config_string = std::fs::read_to_string(&config_filename)
                .map_err(|e| format_err!("Could not read config file at '{}': {}", config_filename.display(), e))?;

        let config = Config::from_str(&config_string)
            .map_err(|e| format_err!("Could not parse config file at '{}': {}", config_filename.display(), e))?; 

        Ok(config)
    }

    fn project_dirs() -> ProjectDirs {
        ProjectDirs::from("", "", "taskerizer").expect("No home directory. Aborting.")
    }

    fn default_config_dir() -> PathBuf {
        Config::project_dirs().config_dir().into()
    }

    fn default_config_filename() -> PathBuf {
        Config::default_config_dir().join("config.toml")
    }

    /// Create config and data directory and write out default config.
    fn write_default_config() -> Result<Config, Error> {
        let default_config = Config::default();
       
        let config_dir = Config::default_config_dir();
        let config_filename = Config::default_config_filename();

        // create directories
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| format_err!("Could not create taskerizer config directory at '{}' when creating initial config file: {}",
                                     &config_dir.display(), e))?;
        std::fs::create_dir_all(&default_config.db_path)
            .map_err(|e| format_err!("Could not create taskerizer config directory at '{}' when creating initial config file: {}",
                                     &config_dir.display(), e))?;

        // write out file
        default_config.write_config(config_filename.as_path())?;

        Ok(default_config)
    }

    fn write_config(&self, config_filename: &Path) -> Result<(), Error> {
        let mut default_file = File::create(&config_filename)
            .map_err(|e| format_err!("Could not open config file at '{}' when creating initial config file: {}",
                                     config_filename.display(), e))?;

        let output = toml::to_string(&self)
            .map_err(|e| format_err!("Could not serialize default config when creating initial config file: {}", e))?;

        default_file.write(output.as_bytes())
            .map_err(|e| format_err!("Could not write out default config when creating initial config file: {}", e))?;
        Ok(())
    }
}

// getters
impl Config {
    /// Get a connection to the database at the location specified by the config file.
    pub fn db(&self) -> Result<SqliteBackend, Error> {
        SqliteBackend::open(&self.db_path)
            .map_err(|e| format_err!("Could not acquire database connection. {}", e))
    }
}


impl FromStr for Config {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let config: Config = toml::from_str(s)
            .map_err(|e| format_err!("Error parsing config file: {}", e))?;

        if config.break_cutoff < 0.0 {
            return Err(format_err!("Parsed break probability was less than 0: {}", config.break_cutoff));
        }
        if config.break_cutoff > 1.0 {
            return Err(format_err!("Parsed break probability was greater than 1: {}", config.break_cutoff));
        }

        Ok(config)
    }
}

// default config
impl Default for Config {
    fn default() -> Config {
        let db_path = Config::project_dirs().data_local_dir().into();
        let break_cutoff = DEFAULT_BREAK_CUTOFF;
        Config {
            db_path,
            break_cutoff,
        }
    }
}

#[cfg(test)]
use tempfile::TempDir;

#[cfg(test)]
impl Config {
    /// Create a `Config` for testing purposes that creates a temporary directory for the database.
    /// The `break_cutoff` is set to `DEFAULT_BREAK_CUTOFF`.
    ///
    /// The TempDir is returned so that the directory can outlive the lifetime of the `Config` -
    /// when the `TempDir` is dropped the Config's database directory is destroyed.
    ///
    // FIXME: wrap the tempdir in a `ConfigTempDir` with a PhantomData<Config> that ties the
    // lifetime of the tempdir to the config.
    //
    // TODO: do the same thing as db::open_test_db and add environment variable that leaks the
    // directory in order to save it for examination. probably need to make wrapper to do this
    pub fn test_config() -> (TempDir, Config) {
        let temp_dir = tempfile::tempdir().expect("Tempdir could not be created");

        let db_path = temp_dir.path().to_path_buf();
        let break_cutoff = DEFAULT_BREAK_CUTOFF;

        (temp_dir, Config {
            db_path,
            break_cutoff,
        })
    }
}
