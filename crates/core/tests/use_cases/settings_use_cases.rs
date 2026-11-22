use lithemux_core::modules::settings::{
    SettingKey, SettingsError, get_config, read_setting, set_config, unset_config, write_setting,
};

use crate::support::in_memory_store::InMemoryStore;

// PLAN §0.6: `lithemux config get|set|unset`
#[test]
fn a_setting_can_be_set_read_and_unset() {
    let store = InMemoryStore::new();
    assert_eq!(get_config(&store, "context_language").unwrap(), None);
    assert_eq!(set_config(&store, "context_language", "ES").unwrap(), "es");
    assert_eq!(
        get_config(&store, "context_language").unwrap().as_deref(),
        Some("es")
    );
    unset_config(&store, "context_language").unwrap();
    assert_eq!(get_config(&store, "context_language").unwrap(), None);
}

// PLAN §0.6: any other key is rejected, naming the valid ones. (`language`
// and `hibernate_after_hours` were the examples here until PLAN §2.6 made
// them settings of the command line too.)
#[test]
fn an_unknown_key_is_rejected_listing_the_valid_ones() {
    let store = InMemoryStore::new();
    for result in [
        get_config(&store, "colour").map(|_| ()),
        set_config(&store, "telemetry", "on").map(|_| ()),
        unset_config(&store, "update_channel"),
    ] {
        let message = result.unwrap_err().to_string();
        assert!(message.contains("claude_path"), "{message}");
        assert!(message.contains("context_language"), "{message}");
    }
}

// PLAN §0.6: values are validated
#[test]
fn an_invalid_value_is_rejected_and_nothing_is_stored() {
    let store = InMemoryStore::new();
    assert!(matches!(
        set_config(&store, "context_language", "fr"),
        Err(SettingsError::InvalidValue { .. })
    ));
    assert!(matches!(
        set_config(&store, "claude_path", "claude"),
        Err(SettingsError::InvalidValue { .. })
    ));
    assert_eq!(get_config(&store, "context_language").unwrap(), None);
    assert_eq!(get_config(&store, "claude_path").unwrap(), None);
}

// PLAN §0.2: a blank setting is the same as no setting
#[test]
fn a_blank_setting_counts_as_unset() {
    let store = InMemoryStore::new();
    write_setting(&store, SettingKey::ClaudePath, "  ").unwrap();
    assert_eq!(read_setting(&store, SettingKey::ClaudePath).unwrap(), None);
}

// PLAN §2.6: `lithemux config` accepts `language`, and what it stores is
// what the app reads.
#[test]
fn the_language_set_from_the_command_line_is_the_one_the_app_uses() {
    use lithemux_core::modules::settings::{InterfaceLanguage, load_preferences};
    let store = InMemoryStore::new();
    assert_eq!(
        load_preferences(&store).language,
        InterfaceLanguage::English
    );
    assert_eq!(set_config(&store, "language", "ES").unwrap(), "es");
    assert_eq!(
        load_preferences(&store).language,
        InterfaceLanguage::Spanish
    );
    unset_config(&store, "language").unwrap();
    assert_eq!(
        load_preferences(&store).language,
        InterfaceLanguage::English
    );
}
