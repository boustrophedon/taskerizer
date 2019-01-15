// dead code allowed because we don't use every example in every test, but each test gets compiled
// separately
#![allow(dead_code)]

use taskerizer_prototype as tkzr;

use tempfile::{tempdir, TempDir};

use self::tkzr::commands::{TKZArgs, TKZCmd};
use self::tkzr::commands::{Add, Current};

use self::tkzr::config::Config;

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

pub fn example_add_cmd_task1() -> TKZArgs {
    let task = "hello this is a task".to_string();
    TKZArgs {
        cmd: Some(TKZCmd::Add( Add {
            reward: false,
            priority: 1,
            task: task,
        })) 
    }  
}

pub fn example_add_cmd_task2() -> TKZArgs {
    let task = "hello this is also a task".to_string();
    TKZArgs {
        cmd: Some(TKZCmd::Add( Add {
            reward: false,
            priority: 9,
            task: task,
        })) 
    }  
}

pub fn example_add_cmd_break1() -> TKZArgs {
    let task = "yo this is a break".to_string();
    TKZArgs {
        cmd: Some(TKZCmd::Add( Add {
            reward: true,
            priority: 2,
            task: task,
        }))
    }
}

pub fn example_add_cmd_break2() -> TKZArgs {
    let task = "ayyy this is another break".to_string();
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

pub fn example_complete() -> TKZArgs {
    TKZArgs {
        cmd: Some(TKZCmd::Complete),
    }
}
