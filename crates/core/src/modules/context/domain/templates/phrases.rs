use crate::modules::session::{ActivityAge, State};

/// One sentence per method. Every sentence states a fact: wording that reads
/// like a system order can trip Claude's injection defenses.
pub trait Phrases {
    fn belongs_to(&self, category_name: &str, root_dir: &str) -> String;
    fn identity(&self, name: &str, role: &str) -> String;

    fn is_mother(&self) -> &'static str;
    fn mother_is(&self, entry: &str) -> String;
    fn no_mother(&self) -> &'static str;

    fn children_heading(&self) -> &'static str;
    fn no_children(&self) -> &'static str;
    fn sisters_heading(&self) -> &'static str;
    fn no_sisters(&self) -> &'static str;
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
    fn child_reporting(&self) -> &'static str;

    fn board_of_mother(&self, board: &str) -> String;
    fn board_of_child(&self, board: &str) -> String;
    fn board_shared(&self, board: &str) -> String;

    fn snapshot_note(&self, category_slug: &str) -> String;
}
