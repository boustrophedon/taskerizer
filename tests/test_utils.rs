// dead code allowed because we don't use every example in every test, but each test gets compiled
// separately
#![allow(dead_code)]

extern crate tempfile;

extern crate taskerizer_prototype as tkzr;

use self::tempfile::{tempdir, TempDir};

use tkzr::commands::{TKZArgs, TKZCmd};
use tkzr::commands::{Add, Current};

use tkzr::config::Config;

/// Create a test config with the database in a temporary directory. We return the TempDir because
/// it is deleted when it is dropped.
pub fn temp_config() -> (TempDir, Config) {
    let test_dir = tempdir().expect("temporary directory could not be created");

    // don't use into_path because test_dir will not be deleted on drop
    let db_path = test_dir.path().to_path_buf();

    let cfg = Config {
        db_path: db_path,
        break_cutoff: 0.33,
    };

    (test_dir, cfg)
}

pub fn example_add_cmd1() -> TKZArgs {
    let task = "hello this is a task".to_string();
    TKZArgs {
        cmd: Some(TKZCmd::Add( Add {
            reward: false,
            priority: 1,
            task: task,
        })) 
    }  
}

pub fn example_add_cmd2() -> TKZArgs {
    let task = "yo this is another task".to_string();
    TKZArgs {
        cmd: Some(TKZCmd::Add( Add {
            reward: true,
            priority: 4,
            task: task,
        }))
    }
}

pub fn example_current() -> TKZArgs {
    TKZArgs {
        cmd: Some(TKZCmd::Current( Current {
            top: false
        })),
    }
}

pub fn example_current_top() -> TKZArgs {
    TKZArgs {
        cmd: Some(TKZCmd::Current( Current {
            top: true
        })),
    }
}
