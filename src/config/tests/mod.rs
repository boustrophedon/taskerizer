use std::str::FromStr;

use crate::config::Config;

mod util;
use util::TempHome;

#[test]
fn test_config_parse_bad_prob_neg() {
    let s = r#"
        db_path = "/tmp/nowhere"
        break_cutoff = -0.5
    "#;

    let res = Config::from_str(&s);
    assert!(res.is_err(), "Invalid break cutoff was parsed correctly: {:?}", res.unwrap());

    let err = res.unwrap_err();
    assert!(err.to_string().contains("Parsed break probability was less than 0"));
}

#[test]
fn test_config_parse_bad_prob_greater_than_1() {
    let s = r#"
        db_path = "/tmp/nowhere"
        break_cutoff = 1.1
    "#;

    let res = Config::from_str(&s);
    assert!(res.is_err(), "Invalid break cutoff was parsed correctly: {:?}", res.unwrap());

    let err = res.unwrap_err();
    assert!(err.to_string().contains("Parsed break probability was greater than 1"));
}

#[test]
/// Make a TempHome, create a config in default directory, check config file is there.
fn test_config_new_in_default() {
    let home = TempHome::new();
    let res = Config::new_in(None);

    assert!(res.is_ok(), "Error creating config: {}", res.unwrap_err());
    assert!(home.temp_home.path().join(".config/taskerizer/config.toml").exists(), "Config path does not exist at expected location.");

    let config = res.unwrap();
    // defaults
    assert!(config.db_path.ends_with(".local/share/taskerizer"));
    assert_eq!(config.break_cutoff, 0.35);
}

#[test]
/// Make a TempHome, create a config in default directory, modify config, read it back, check the
/// modification is there.
fn test_config_new_in_default_existing_config() {
    let home = TempHome::new();

    { // make the original config
    let res = Config::new_in(None);
    let config_path = home.temp_home.path().join(".config/taskerizer/config.toml");
    assert!(res.is_ok(), "Error creating config: {}", res.unwrap_err());
    assert!(config_path.exists(), "Config path does not exist at expected location.");

    let mut config = res.unwrap();
    config.break_cutoff = 0.9;
    config.write_config(&config_path).expect("Failed to write out config");
    }

    // read back the modified config
    let res = Config::new_in(None);
    assert!(res.is_ok(), "Error opening config: {}", res.unwrap_err());
    let config = res.unwrap();

    // default db path, different break_cutoff
    assert!(config.db_path.ends_with(".local/share/taskerizer"));
    assert_eq!(config.break_cutoff, 0.9);
}

#[test]
/// Make a TempHome, try to open a config in custom, non-existant directory, check error.
fn test_config_new_in_custom_no_path() {
    let home = TempHome::new();
    let dir = home.temp_home.path();
    let res = Config::new_in(Some(dir.join("taskerizer_custom/custom_config.toml")));

    assert!(res.is_err(), "No error creating config: {:?}", res.unwrap());
    let err = res.unwrap_err();
    assert!(err.to_string().contains("Config file path was given but there was no config file there"),
            "Incorrect error message when trying to pass non-existant custom config file: {}", err);
}

#[test]
/// Make a TempHome, create a config in another directory, check config file is parsed.
fn test_config_new_in_custom_exists() {
    let home = TempHome::new();
    let dir = home.temp_home.path();
    let config_filename = dir.join("taskerizer_custom/custom_config");

    // create custom config in custom dir
    std::fs::create_dir(dir.join("taskerizer_custom")).expect("Failed to create custom config dir");

    let custom_config = util::example_custom_config();
    custom_config.write_config(&config_filename).expect("failed to write out config");


    // parse the config we wrote out
    let res = Config::new_in(Some(config_filename.into()));

    assert!(res.is_ok(), "Error opening config: {}", res.unwrap_err());
    assert!(!dir.join(".config/taskerizer").exists(),
        "Default config directory was created even though we used a custom config");

    let config = res.unwrap();
    // custom values as above
    assert_eq!(config.db_path, custom_config.db_path);
    assert_eq!(config.break_cutoff, custom_config.break_cutoff);
}
