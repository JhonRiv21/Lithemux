use std::path::PathBuf;

use lithemux_core::modules::category::{CategoryError, InvalidSlug};
use lithemux_core::modules::launch::LaunchError;
use lithemux_core::modules::session::SessionError;
use lithemux_core::modules::workspace::WorkspaceError;

use crate::i18n::{Language, Message, Texts};
use crate::session_host::{Failure, HostError};
use crate::sidebar::error::SidebarError;

fn every_problem() -> Vec<Message> {
    let mother = SessionError::MotherExists {
        category: "demo".into(),
        mother: "planificadora".into(),
    };
    let taken = SessionError::MessagingNameTaken {
        name: "demo-api".into(),
        owner: "demo/api".into(),
    };
    vec![
        (&SidebarError::UnusableName("¡¡".into())).into(),
        (&SidebarError::Gone).into(),
        (&SidebarError::Session(mother)).into(),
        (&SidebarError::Session(taken)).into(),
        (&SidebarError::Session(SessionError::AlreadyExists("demo/api".into()))).into(),
        (&SidebarError::Session(SessionError::NotFound("demo/api".into()))).into(),
        (&SidebarError::Session(SessionError::InvalidSlug(InvalidSlug("A b".into())))).into(),
        (&SidebarError::Category(CategoryError::AlreadyExists("demo".into()))).into(),
        (&SidebarError::Category(CategoryError::EmptyName)).into(),
        (&SidebarError::Category(CategoryError::NotADirectory(PathBuf::from("/nowhere")))).into(),
        (&Failure::ClaudeNotFound("not in the PATH".into())).into(),
        (&Failure::MissingFolder("/gone".into())).into(),
        (&Failure::HookMissing("/app/lithemux-hook".into())).into(),
        (&Failure::CouldNotStart("/bin/claude".into())).into(),
        (&Failure::Other("disk full".into())).into(),
        (&HostError::Launch(LaunchError::AlreadyOpen { pid: Some(7) })).into(),
        (&HostError::Session(SessionError::NotFound("demo/x".into()))).into(),
        (&WorkspaceError::TooManyPanes).into(),
        (&WorkspaceError::UnknownPane(9)).into(),
    ]
}

// Audit of block 3 (7), PLAN §2.6: what goes wrong is said with a text of
// the locales, in both languages, with every value in its place.
#[test]
fn every_problem_has_its_text_in_both_languages() {
    for language in [Language::English, Language::Spanish] {
        let texts = Texts::of(language);
        for message in every_problem() {
            let said = texts.say(&message);
            assert_ne!(said, message.key, "{language:?}");
            assert!(!said.contains('{'), "{language:?}: {said}");
            assert!(
                language == Language::English || texts.is_translated(message.key),
                "{}",
                message.key
            );
        }
    }
}

// Audit of block 3 (7): the mother rule is said in the user's language.
#[test]
fn a_second_mother_is_refused_in_the_language_of_the_interface() {
    let error = SidebarError::Session(SessionError::MotherExists {
        category: "demo".into(),
        mother: "planificadora".into(),
    });
    let message = Message::from(&error);
    assert_eq!(
        Texts::of(Language::Spanish).say(&message),
        "«demo» ya tiene una sesión madre (planificadora)."
    );
    assert_eq!(
        Texts::of(Language::English).say(&message),
        "“demo” already has a mother session (planificadora)."
    );
}

// Audit of block 3 (7): why a session did not start, on its card.
#[test]
fn a_failed_start_is_worded_from_the_locales() {
    use crate::sidebar::status::Status;
    use crate::sidebar::status_text::status_text;
    let status = Status::Failed(Failure::MissingFolder("/work/gone".into()));
    assert_eq!(
        status_text(&status, &Texts::of(Language::Spanish)),
        "no se pudo iniciar: la carpeta /work/gone no existe"
    );
}

// Audit of block 3 (7): «Claude is waiting for your input» on the card.
#[test]
fn a_waiting_card_says_that_claude_waits_for_the_user() {
    use crate::sidebar::status::Status;
    use crate::sidebar::status_text::card_status_text;
    assert_eq!(
        card_status_text(&Status::Waiting, &Texts::of(Language::English)),
        "Claude is waiting for your input"
    );
    assert_eq!(
        card_status_text(&Status::Idle, &Texts::of(Language::English)),
        "idle"
    );
}
