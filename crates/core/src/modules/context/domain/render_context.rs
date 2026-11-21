use crate::modules::category::Category;
use crate::modules::context::domain::context_input::{ContextInput, MAX_CONTEXT_CHARS};
use crate::modules::context::domain::lang::Lang;
use crate::modules::context::domain::role_text::role_text;
use crate::modules::context::domain::templates::en::En;
use crate::modules::context::domain::templates::es::Es;
use crate::modules::context::domain::templates::phrases::Phrases;
use crate::modules::session::{Kind, Profile, Session};

/// Renders the text a session receives at startup. Stays under
/// `MAX_CONTEXT_CHARS` by listing fewer sessions when needed.
pub fn render_context(input: &ContextInput<'_>) -> String {
    let phrases: &dyn Phrases = match input.lang {
        Lang::En => &En,
        Lang::Es => &Es,
    };
    let mut shown = input.others.len();
    loop {
        let text = compose(input, phrases, shown);
        if text.chars().count() < MAX_CONTEXT_CHARS || shown == 0 {
            return text;
        }
        shown -= 1;
    }
}

fn compose(input: &ContextInput<'_>, phrases: &dyn Phrases, shown: usize) -> String {
    let ContextInput {
        category, me, now, ..
    } = input;
    let board = input.board.display().to_string();
    let mother = input.others.iter().find(|s| s.kind == Kind::Mother);
    let rest: Vec<&Session> = input
        .others
        .iter()
        .filter(|s| s.kind != Kind::Mother)
        .collect();
    let entry = |session: &Session| describe(phrases, category, session, *now);
    let list = |heading: &str, empty: &str| -> Vec<String> {
        if rest.is_empty() {
            return vec![empty.to_string()];
        }
        let mut block = vec![heading.to_string()];
        block.extend(rest.iter().take(shown).map(|s| format!("- {}", entry(s))));
        let hidden = rest.len().saturating_sub(shown);
        if hidden > 0 {
            block.push(format!("- {}", phrases.more(hidden)));
        }
        block
    };

    let mut lines = vec![
        phrases.belongs_to(&category.name, &category.root_dir),
        phrases.identity(&me.messaging_name(category), &role_text(&me.role)),
    ];
    if me.kind == Kind::Mother {
        lines.push(phrases.is_mother().to_string());
        lines.extend(list(phrases.children_heading(), phrases.no_children()));
        lines.push(phrases.messaging().to_string());
        lines.push(phrases.board_of_mother(&board));
    } else if let Some(mother) = mother {
        lines.push(phrases.mother_is(&entry(mother)));
        lines.extend(list(phrases.sisters_heading(), phrases.no_sisters()));
        lines.push(format!(
            "{} {}",
            phrases.messaging(),
            phrases.child_reporting()
        ));
        lines.push(phrases.board_of_child(&board));
    } else {
        lines.push(phrases.no_mother().to_string());
        lines.extend(list(phrases.sisters_heading(), phrases.no_sisters()));
        lines.push(phrases.messaging().to_string());
        lines.push(phrases.board_shared(&board));
    }
    lines.push(phrases.snapshot_note(&category.slug));
    lines.join("\n")
}

fn describe(phrases: &dyn Phrases, category: &Category, session: &Session, now: i64) -> String {
    phrases.entry(
        &session.messaging_name(category),
        &role_text(&session.role),
        session.state,
        session.last_activity_at.map(|at| (now - at).max(0)),
        session.profile == Profile::Shell,
    )
}
