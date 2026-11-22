use crate::modules::category::Category;
use crate::modules::context::domain::context_input::{ContextInput, MAX_CONTEXT_CHARS};
use crate::modules::context::domain::lang::Lang;
use crate::modules::context::domain::role_text::role_text;
use crate::modules::context::domain::templates::en::En;
use crate::modules::context::domain::templates::es::Es;
use crate::modules::context::domain::templates::phrases::Phrases;
use crate::modules::context::domain::user_text::one_line;
use crate::modules::session::{ActivityAge, Kind, Profile, Session};

const MAX_NAME_CHARS: usize = 80;
const MAX_PATH_CHARS: usize = 240;

/// What every version of the text shares, with the user's text already
/// flattened.
struct Scene<'a> {
    input: &'a ContextInput<'a>,
    phrases: &'a dyn Phrases,
    category_name: String,
    root_dir: String,
    board: String,
}

/// Renders the text a session receives at startup. Stays under
/// `MAX_CONTEXT_CHARS` by listing fewer sessions when needed. With no session
/// listed it always fits: every text of the user is capped, and so are the
/// slugs (the test `the_largest_context_without_a_list_fits` keeps that true).
pub fn render_context(input: &ContextInput<'_>) -> String {
    let scene = Scene {
        input,
        phrases: match input.lang {
            Lang::En => &En,
            Lang::Es => &Es,
        },
        category_name: one_line(&input.category.name, MAX_NAME_CHARS),
        root_dir: one_line(&input.category.root_dir, MAX_PATH_CHARS),
        board: one_line(&input.board.display().to_string(), MAX_PATH_CHARS),
    };
    let mut shown = input.others.len();
    loop {
        let text = compose(&scene, shown);
        if text.chars().count() < MAX_CONTEXT_CHARS || shown == 0 {
            return text;
        }
        shown -= 1;
    }
}

fn compose(scene: &Scene<'_>, shown: usize) -> String {
    let Scene { input, phrases, .. } = scene;
    let (category, me) = (input.category, input.me);
    let mut lines = vec![
        phrases.belongs_to(&scene.category_name, &scene.root_dir),
        phrases.identity(&me.messaging_name(category), &role_text(&me.role)),
    ];
    lines.extend(family(scene, shown));
    lines.push(phrases.snapshot_note(&category.slug));
    lines.join("\n")
}

/// Who the session works with, how they talk and who keeps the board: one
/// wording for a coordinator, one for the sessions it coordinates and one for
/// sessions without a coordinator.
fn family(scene: &Scene<'_>, shown: usize) -> Vec<String> {
    let Scene {
        input,
        phrases,
        board,
        ..
    } = scene;
    let coordinator = input.others.iter().find(|s| s.kind == Kind::Coordinator);
    let mut lines = Vec::new();
    if input.me.kind == Kind::Coordinator {
        lines.push(phrases.is_coordinator().to_string());
        lines.extend(others(
            scene,
            phrases.members_heading(),
            phrases.no_members(),
            shown,
        ));
        lines.push(phrases.messaging().to_string());
        lines.push(phrases.board_of_coordinator(board));
    } else if let Some(coordinator) = coordinator {
        lines.push(phrases.coordinator_is(&describe(scene, coordinator)));
        lines.extend(others(
            scene,
            phrases.others_heading(),
            phrases.no_others(),
            shown,
        ));
        let reporting = phrases.member_reporting();
        lines.push(format!("{} {reporting}", phrases.messaging()));
        lines.push(phrases.board_of_member(board));
    } else {
        lines.push(phrases.no_coordinator().to_string());
        lines.extend(others(
            scene,
            phrases.others_heading(),
            phrases.no_others(),
            shown,
        ));
        lines.push(phrases.messaging().to_string());
        lines.push(phrases.board_shared(board));
    }
    lines
}

/// The sessions that are not the coordinator: the first `shown`, then how many
/// were left out.
fn others(scene: &Scene<'_>, heading: &str, empty: &str, shown: usize) -> Vec<String> {
    let rest: Vec<&Session> = scene
        .input
        .others
        .iter()
        .filter(|s| s.kind != Kind::Coordinator)
        .collect();
    if rest.is_empty() {
        return vec![empty.to_string()];
    }
    let mut block = vec![heading.to_string()];
    block.extend(
        rest.iter()
            .take(shown)
            .map(|session| format!("- {}", describe(scene, session))),
    );
    let hidden = rest.len().saturating_sub(shown);
    if hidden > 0 {
        block.push(format!("- {}", scene.phrases.more(hidden)));
    }
    block
}

fn describe(scene: &Scene<'_>, session: &Session) -> String {
    let category: &Category = scene.input.category;
    scene.phrases.entry(
        &session.messaging_name(category),
        &role_text(&session.role),
        session.state,
        session
            .last_activity_at
            .map(|at| ActivityAge::between(scene.input.now, at)),
        session.profile == Profile::Shell,
    )
}
