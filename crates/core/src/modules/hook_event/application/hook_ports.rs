use crate::modules::context::ContextPorts;
use crate::modules::hook_event::ports::app_notifier::AppNotifier;
use crate::modules::hook_event::ports::hook_log::HookLog;

pub struct HookPorts<'a> {
    pub context: ContextPorts<'a>,
    pub log: &'a dyn HookLog,
    pub notifier: &'a dyn AppNotifier,
}
