use crate::modules::category::{BoardStore, CategoryRepository};
use crate::modules::session::SessionRepository;
use crate::modules::settings::SettingsRepository;

pub struct ContextPorts<'a> {
    pub categories: &'a dyn CategoryRepository,
    pub sessions: &'a dyn SessionRepository,
    pub settings: &'a dyn SettingsRepository,
    pub boards: &'a dyn BoardStore,
}
