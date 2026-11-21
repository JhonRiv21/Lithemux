use lithemux_core::modules::settings::SettingsRepository;

/// What every SettingsRepository must do. PLAN §0.2.
pub fn settings_repository_contract(settings: &dyn SettingsRepository) {
    assert_eq!(settings.get("language").unwrap(), None);
    settings.set("language", "es").unwrap();
    assert_eq!(settings.get("language").unwrap().as_deref(), Some("es"));
    settings.set("language", "en").unwrap();
    assert_eq!(settings.get("language").unwrap().as_deref(), Some("en"));
    settings.remove("language").unwrap();
    assert_eq!(settings.get("language").unwrap(), None);
    settings.remove("language").unwrap();
}
