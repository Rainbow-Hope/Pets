use desktop_pets::config::{AppConfig, LoadOutcome, load_or_recover, save_atomic};
use std::fs;

#[test]
fn missing_configuration_is_created_with_safe_defaults() {
    let directory = tempfile::tempdir().expect("temp dir");
    let path = directory.path().join("config.json");

    let outcome = load_or_recover(&path).expect("create default config");

    assert_eq!(outcome, LoadOutcome::CreatedDefault(AppConfig::default()));
    assert!(path.exists());
}

#[test]
fn malformed_configuration_is_preserved_before_recovery() {
    let directory = tempfile::tempdir().expect("temp dir");
    let path = directory.path().join("config.json");
    fs::write(&path, "{not-json").expect("write invalid file");

    let outcome = load_or_recover(&path).expect("recover config");

    let backup = match outcome {
        LoadOutcome::Recovered { backup, config } => {
            assert_eq!(config, AppConfig::default());
            backup
        }
        other => panic!("expected recovery, got {other:?}"),
    };
    assert_eq!(
        fs::read_to_string(backup).expect("read backup"),
        "{not-json"
    );
    assert!(path.exists());
}

#[test]
fn atomic_save_does_not_leave_a_temporary_file() {
    let directory = tempfile::tempdir().expect("temp dir");
    let path = directory.path().join("config.json");

    save_atomic(&path, &AppConfig::default()).expect("save");

    assert!(!directory.path().join("config.json.tmp").exists());
    let loaded: AppConfig =
        serde_json::from_slice(&fs::read(path).expect("read")).expect("valid json");
    assert_eq!(loaded, AppConfig::default());
}
