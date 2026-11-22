use crate::modules::context::domain::templates::phrases::Phrases;
use crate::modules::session::{ActivityAge, State};

pub struct En;

impl En {
    fn last_activity(age: Option<ActivityAge>) -> String {
        match age {
            Some(age) => format!("last activity {age}"),
            None => "no activity yet".to_string(),
        }
    }
}

impl Phrases for En {
    fn belongs_to(&self, category_name: &str, root_dir: &str) -> String {
        format!(
            "lithemux: this session belongs to the category \"{category_name}\" (root folder {root_dir})."
        )
    }

    fn identity(&self, name: &str, role: &str) -> String {
        format!("This session's name: {name}. Role: {role}.")
    }

    fn is_mother(&self) -> &'static str {
        "This session is the mother of the category: it coordinates the category and is the only one that writes the board."
    }

    fn mother_is(&self, entry: &str) -> String {
        format!("Mother of the category: {entry}.")
    }

    fn no_mother(&self) -> &'static str {
        "The category has no mother: its sessions are sisters and coordinate directly with each other."
    }

    fn children_heading(&self) -> &'static str {
        "Children:"
    }

    fn no_children(&self) -> &'static str {
        "The category has no child sessions yet."
    }

    fn sisters_heading(&self) -> &'static str {
        "Sisters:"
    }

    fn no_sisters(&self) -> &'static str {
        "This session has no sisters yet."
    }

    fn more(&self, hidden: usize) -> String {
        format!("… and {hidden} more")
    }

    fn entry(
        &self,
        name: &str,
        role: &str,
        state: State,
        age: Option<ActivityAge>,
        shell: bool,
    ) -> String {
        let shell = if shell {
            "shell terminal, not reachable with SendMessage; "
        } else {
            ""
        };
        format!(
            "{name} ({shell}role: {role}; state: {}, {})",
            state.as_str(),
            Self::last_activity(age)
        )
    }

    fn messaging(&self) -> &'static str {
        "Sessions talk to each other with SendMessage using those names."
    }

    fn child_reporting(&self) -> &'static str {
        "When this session finishes a task or gets blocked, it reports to the mother; anything outside its role goes through the mother."
    }

    fn board_of_mother(&self, board: &str) -> String {
        format!("Category board (written only by this session): {board}")
    }

    fn board_of_child(&self, board: &str) -> String {
        format!(
            "Category board (kept by the mother; read-only for this session unless the mother says otherwise): {board}"
        )
    }

    fn board_shared(&self, board: &str) -> String {
        format!("Category board (shared by the sisters): {board}")
    }

    fn snapshot_note(&self, category_slug: &str) -> String {
        format!(
            "This list is a snapshot taken at startup; `lithemux status {category_slug}` shows the current state."
        )
    }
}
