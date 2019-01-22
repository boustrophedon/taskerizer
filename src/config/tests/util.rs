use std::env;
use std::path::PathBuf;

use tempfile::{tempdir, TempDir};

use super::super::Config;

pub fn example_custom_config() -> Config {
    Config {
        db_path: PathBuf::from("/tmp/nowhere"),
        break_cutoff: 0.1,
    }
}

// I didn't want to pull in lazy-static just for this one thing so here's a tiny mutex.
use std::sync::atomic::{AtomicBool, Ordering};

// false -> not in use, true -> in use
static ENV_VAR_LOCK: AtomicBool = AtomicBool::new(false);
struct EnvVarMutex;
impl EnvVarMutex {
    pub fn acquire() -> EnvVarMutex {
        while ENV_VAR_LOCK.compare_and_swap(false, true, Ordering::SeqCst) {}
        EnvVarMutex
    }
}
impl Drop for EnvVarMutex {
    fn drop(&mut self) {
        assert!(ENV_VAR_LOCK.compare_and_swap(true, false, Ordering::SeqCst), "drop unlocked mutex");
    }
}


pub struct TempHome {
    pub original_home: String,
    pub temp_home: TempDir,

    pub original_config_dir: Option<String>,
    pub original_data_dir: Option<String>,
    _env_var_mutex: EnvVarMutex,
}

impl TempHome {
    pub fn new() -> TempHome {
        let env_var_mutex = EnvVarMutex::acquire();
        TempHome::with_lock(env_var_mutex)
    }

    fn with_lock(_env_var_mutex: EnvVarMutex) -> TempHome {
        let original_home = env::var("HOME").expect("could not get home dir");
        let original_config_dir = env::var("XDG_CONFIG_HOME").ok();
        let original_data_dir = env::var("XDG_DATA_HOME").ok();

        let temp_home = tempdir().expect("Could not create temp dir");

        env::set_var("HOME", temp_home.path());

        let config_path = temp_home.path().join(".config");
        std::fs::create_dir_all(&config_path).expect("Could not create temp home config path");
        env::set_var("XDG_CONFIG_HOME", config_path);

        let data_path = temp_home.path().join(".local/share");
        std::fs::create_dir_all(&data_path).expect("Could not create temp home data path");
        env::set_var("XDG_DATA_HOME", data_path);

        TempHome {
            original_home,
            temp_home,
            original_config_dir,
            original_data_dir,
            _env_var_mutex,
        }
    }
}

impl Drop for TempHome {
    fn drop(&mut self) {
        env::set_var("HOME", &self.original_home);
        if let Some(config_dir) = &self.original_config_dir {
            env::set_var("XDG_CONFIG_HOME", config_dir);
        }
        if let Some(data_dir) = &self.original_data_dir {
            env::set_var("XDG_DATA_HOME", data_dir);
        }

    }
}

#[cfg(target_os = "linux")]
#[test]
fn test_config_temp_home() {
    // we have to acquire the env lock here separately or it could happen that this test starts
    // running in the middle of another config test, and then the "original_home" is actually a
    // temp home.
    //
    // similarly, the current_home at the bottom could begin when another config test's temphome is
    // active because the lock was dropped along with this test's temphome in the inner scope.
    let env_lock = EnvVarMutex::acquire();
    let original_home = env::var("HOME").expect("could not get home dir");
    {
        let _temp_home = TempHome::with_lock(env_lock);
        let current_home = env::var("HOME").expect("could not get home dir");
        assert!(original_home != current_home);
        assert!(current_home.starts_with("/tmp"),
            "Temp home did not start with /tmp: {:?}", current_home);

    }
    let current_home = env::var("HOME").expect("could not get home dir");
    assert_eq!(original_home, current_home);
}

#[cfg(target_os = "linux")]
#[test]
fn test_config_directories_with_tmp_home() {
    let _tmp_home = TempHome::new();

    let proj = Config::project_dirs();

    let config = proj.config_dir();
    let data = proj.data_dir();
    assert!(config.starts_with("/tmp"),
        "Temp home config dir did not start with /tmp: {:?}", config);
    assert!(data.starts_with("/tmp"),
        "Temp home data dir did not start with /tmp: {:?}", data);

    assert!(config.to_str().unwrap().contains("taskerizer"),
        "Temp home config dir did not contain string \"taskerizer\": {:?}", config);
    assert!(data.to_str().unwrap().contains("taskerizer"),
        "Temp home data dir did not contain string \"taskerizer\": {:?}", data);
}
