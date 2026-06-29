use desktop_pets::config::{
    AppConfig, ConfigError, MovementMode, SavedInstance, StartupPolicy, validate_size,
};

#[test]
fn fresh_config_contains_one_rainbow_hope_pet() {
    let config = AppConfig::default();

    assert_eq!(config.schema_version, 1);
    assert_eq!(config.startup, StartupPolicy::None);
    assert_eq!(config.instances.len(), 1);
    assert_eq!(config.instances[0].pet_id, "rainbow-hope");
    assert_eq!(config.instances[0].size_percent, 100);
    assert_eq!(config.instances[0].movement, MovementMode::Fixed);
    assert_eq!(config.last_active, Some(config.instances[0].id));
}

#[test]
fn size_accepts_only_twenty_through_two_hundred_percent() {
    assert_eq!(validate_size(20), Ok(20));
    assert_eq!(validate_size(200), Ok(200));
    assert_eq!(validate_size(19), Err(ConfigError::InvalidSize(19)));
    assert_eq!(validate_size(201), Err(ConfigError::InvalidSize(201)));
}

#[test]
fn configuration_round_trips_without_losing_instance_identity() {
    let mut config = AppConfig::default();
    let original_id = config.instances[0].id;
    config.startup = StartupPolicy::Specific(original_id);
    config.instances.push(SavedInstance::default());

    let json = serde_json::to_string_pretty(&config).expect("serialize config");
    let loaded: AppConfig = serde_json::from_str(&json).expect("deserialize config");

    assert_eq!(loaded, config);
    assert_eq!(loaded.instances[0].id, original_id);
}
