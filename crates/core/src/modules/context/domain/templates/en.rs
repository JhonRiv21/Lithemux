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

    fn is_coordinator(&self) -> &'static str {
        "This session is the coordinator session of the category: it coordinates the category and is the only one that writes the board."
    }

    fn coordinator_is(&self, entry: &str) -> String {
        format!("Coordinator of the category: {entry}.")
    }

    fn no_coordinator(&self) -> &'static str {
        "The category has no coordinator session: its sessions coordinate directly with each other."
    }

    fn members_heading(&self) -> &'static str {
        "The other sessions:"
    }

    fn no_members(&self) -> &'static str {
        "The category has no other sessions yet."
    }

    fn others_heading(&self) -> &'static str {
        "The other sessions:"
    }

    fn no_others(&self) -> &'static str {
        "The category has no other sessions yet."
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

    fn member_reporting(&self) -> &'static str {
        "When this session finishes a task or gets blocked, it reports to the coordinator; anything outside its role goes through the coordinator."
    }

    fn board_of_coordinator(&self, board: &str) -> String {
        format!("Category board (written only by this session): {board}")
    }

    fn board_of_member(&self, board: &str) -> String {
        format!(
            "Category board (kept by the coordinator; read-only for this session unless the coordinator says otherwise): {board}"
        )
    }

    fn board_shared(&self, board: &str) -> String {
        format!("Category board (shared by its sessions): {board}")
    }

    fn snapshot_note(&self, category_slug: &str) -> String {
        format!(
            "This list is a snapshot taken at startup; `lithemux status {category_slug}` shows the current state."
        )
    }
}
