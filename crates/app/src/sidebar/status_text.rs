//! A status in words and in color.

use super::status::Status;
use crate::i18n::Texts;
use crate::theme;

pub fn status_text(status: &Status, texts: &Texts) -> String {
    match status {
        Status::Opening => texts.get("status.opening"),
        Status::Closing => texts.get("status.closing"),
        Status::Working => texts.get("status.working"),
        Status::Idle => texts.get("status.idle"),
        Status::Waiting => texts.get("status.waiting"),
        Status::Hibernated => texts.get("status.hibernated"),
        Status::Ended => texts.get("status.ended"),
        Status::NotStarted => texts.get("status.new"),
        Status::Running => texts.get("status.running"),
        Status::NotRunning => texts.get("status.not_running"),
        Status::OpenElsewhere(None) => texts.get("status.open_elsewhere"),
        Status::OpenElsewhere(Some(pid)) => {
            texts.with("status.open_elsewhere_pid", &[("pid", &pid.to_string())])
        }
        Status::Failed(failure) => {
            texts.with("status.failed", &[("reason", &texts.say(&failure.into()))])
        }
    }
}

/// On a card, waiting is said in full: it is the one that needs the user.
pub fn card_status_text(status: &Status, texts: &Texts) -> String {
    match status {
        Status::Waiting => texts.get("status.waiting_detail"),
        other => status_text(other, texts),
    }
}

pub fn status_color(status: &Status) -> u32 {
    match status {
        Status::Waiting => theme::ATTENTION,
        Status::Working | Status::Opening => theme::WORKING,
        Status::Idle | Status::Running => theme::ACCENT,
        Status::Failed(_) => theme::DANGER,
        Status::Closing
        | Status::Hibernated
        | Status::Ended
        | Status::NotStarted
        | Status::NotRunning
        | Status::OpenElsewhere(_) => theme::TEXT_FAINT,
    }
}
