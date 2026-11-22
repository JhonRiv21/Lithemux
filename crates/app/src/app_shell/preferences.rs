//! The settings in the running app: read from the database, and applied
//! the moment they change, from the settings window or from
//! `lithemux config` in a terminal. Nothing here needs a restart.

use std::rc::Rc;

use gpui::{App, Context, WindowAppearance};
use lithemux_core::modules::settings::{
    Preferences, SettingKey, SqliteSettingsRepository, load_preferences, set_config, unset_config,
};

use super::live::Live;
use crate::i18n::{Language, Message, Texts};
use crate::terminal_view::{DEFAULT_FONT_FAMILY, set_look};
use crate::theme;

fn system_is_dark(cx: &App) -> bool {
    matches!(
        cx.window_appearance(),
        WindowAppearance::Dark | WindowAppearance::VibrantDark
    )
}

impl Live {
    pub fn preferences(&self) -> &Preferences {
        &self.preferences
    }

    /// Reads the settings again and applies them: language, theme, and the
    /// look of the terminals. Returns whether anything on screen changed.
    pub(super) fn follow_the_settings(&mut self, cx: &App) -> bool {
        let preferences = load_preferences(&SqliteSettingsRepository::new(&self.db));
        let is_light = theme::is_light_for(preferences.theme, system_is_dark(cx));
        let theme_changed = theme::use_light(is_light);
        let family = self.usable_font(&preferences.font_family, cx);
        let look_changed = set_look(&family, preferences.font_size, preferences.scrollback);
        let language = Language::from(preferences.language);
        let language_changed = self.texts.language() != language;
        if language_changed {
            self.texts = Rc::new(Texts::of(language));
        }
        let changed = preferences != self.preferences;
        self.preferences = preferences;
        changed || theme_changed || look_changed || language_changed
    }

    /// The font to draw with: the one asked for, or the default when the
    /// machine does not have it (it would fall back to whatever the system
    /// picks). Listing the fonts of the machine is slow, and this runs
    /// every few seconds: the answer is kept until another font is asked.
    fn usable_font(&mut self, asked: &str, cx: &App) -> String {
        if let Some((last_asked, usable)) = &self.checked_font
            && last_asked == asked
        {
            return usable.clone();
        }
        let is_installed = cx
            .text_system()
            .all_font_names()
            .iter()
            .any(|name| name == asked);
        let usable = if is_installed {
            asked.to_string()
        } else {
            DEFAULT_FONT_FAMILY.to_string()
        };
        self.checked_font = Some((asked.to_string(), usable.clone()));
        usable
    }

    /// Changes a setting, as `lithemux config set` would, and applies it.
    /// A value that is not accepted leaves a notice and changes nothing.
    pub fn set_setting(&mut self, key: SettingKey, value: &str, cx: &mut Context<Self>) -> bool {
        let settings = SqliteSettingsRepository::new(&self.db);
        let stored = if value.trim().is_empty() {
            unset_config(&settings, key.as_str())
        } else {
            set_config(&settings, key.as_str(), value).map(|_| ())
        };
        let accepted = stored.is_ok();
        self.notice = stored.err().map(|_| {
            let message = Message::with("error.invalid_setting", "value", value.trim());
            self.texts.say(&message)
        });
        self.reload(cx);
        cx.notify();
        accepted
    }
}
