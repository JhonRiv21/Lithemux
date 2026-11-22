use crate::modules::session::{ActivityAge, State};

/// One sentence per method. Every sentence states a fact: wording that reads
/// like a system order can trip Claude's injection defenses.
pub trait Phrases {
    fn belongs_to(&self, category_name: &str, root_dir: &str) -> String;
    fn identity(&self, name: &str, role: &str) -> String;

    fn is_coordinator(&self) -> &'static str;
    fn coordinator_is(&self, entry: &str) -> String;
    fn no_coordinator(&self) -> &'static str;

    fn members_heading(&self) -> &'static str;
    fn no_members(&self) -> &'static str;
    fn others_heading(&self) -> &'static str;
    fn no_others(&self) -> &'static str;
    fn more(&self, hidden: usize) -> String;

    /// `age` is `None` for a session that was never active.
    fn entry(
        &self,
        name: &str,
        role: &str,
        state: State,
        age: Option<ActivityAge>,
        shell: bool,
    ) -> String;

    fn messaging(&self) -> &'static str;
    /// How a session reports to its coordinator, by its address.
    fn member_reporting(&self, coordinator: &str) -> String;
    /// How the coordinator hands out a task.
    fn coordinator_assigning(&self) -> &'static str;

    fn board_of_coordinator(&self, board: &str) -> String;
    fn board_of_member(&self, board: &str) -> String;
    fn board_shared(&self, board: &str) -> String;

    fn snapshot_note(&self, category_slug: &str) -> String;
}
