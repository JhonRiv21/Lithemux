use lithemux_core::modules::settings::{get_config, set_config, unset_config};

use crate::output;
use crate::wiring::Wiring;

pub fn get(key: &str) -> anyhow::Result<()> {
    let wiring = Wiring::open()?;
    output::config::value(get_config(&wiring.settings(), key)?.as_deref());
    Ok(())
}

pub fn set(key: &str, value: &str) -> anyhow::Result<()> {
    let wiring = Wiring::open()?;
    let stored = set_config(&wiring.settings(), key, value)?;
    output::config::changed(key, Some(&stored));
    Ok(())
}

pub fn unset(key: &str) -> anyhow::Result<()> {
    let wiring = Wiring::open()?;
    unset_config(&wiring.settings(), key)?;
    output::config::changed(key, None);
    Ok(())
}
