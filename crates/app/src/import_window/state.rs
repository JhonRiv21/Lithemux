//! The import as the user edits it: which sessions are marked and where
//! each one goes. No gpui.

use std::path::Path;

use lithemux_core::modules::category::Category;
use lithemux_core::modules::import::{Chosen, Destination, Proposal, ProposedSession};

/// One place a session can go, as the selector lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    /// The category's name; for a new one, the name it will get.
    pub name: String,
    pub is_new: bool,
    pub destination: Destination,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub session: ProposedSession,
    pub marked: bool,
    /// `None` until the user chooses, for a session of the home folder.
    pub destination: Option<Destination>,
}

pub struct ImportForm {
    pub rows: Vec<Row>,
    existing: Vec<Choice>,
    proposed_new: Vec<Choice>,
}

/// The name a new category for `folder` gets: the folder's own.
fn folder_name(folder: &str) -> String {
    Path::new(folder)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| folder.to_string())
}

impl ImportForm {
    /// Marked as proposed: what has a saved conversation.
    pub fn new(proposal: &Proposal, categories: &[Category]) -> Self {
        let rows = proposal
            .sessions
            .iter()
            .map(|session| Row {
                session: session.clone(),
                marked: session.selected,
                destination: match &session.destination {
                    Destination::Unassigned => None,
                    destination => Some(destination.clone()),
                },
            })
            .collect();
        let existing = categories
            .iter()
            .map(|category| Choice {
                name: category.name.clone(),
                is_new: false,
                destination: Destination::Existing {
                    category_id: category.id.clone(),
                },
            })
            .collect();
        let proposed_new = proposal
            .new_categories
            .iter()
            .map(|category| Choice {
                name: category.name.clone(),
                is_new: true,
                destination: Destination::New {
                    root_dir: category.root_dir.clone(),
                },
            })
            .collect();
        ImportForm {
            rows,
            existing,
            proposed_new,
        }
    }

    fn row_mut(&mut self, number: usize) -> Option<&mut Row> {
        self.rows
            .iter_mut()
            .find(|row| row.session.number == number)
    }

    pub fn toggle(&mut self, number: usize) {
        if let Some(row) = self.row_mut(number) {
            row.marked = !row.marked;
        }
    }

    pub fn set_destination(&mut self, number: usize, destination: Destination) {
        if let Some(row) = self.row_mut(number) {
            row.destination = Some(destination);
        }
    }

    /// Where the session `number` can go: the categories there are, the new
    /// ones the proposal makes, and a new one for its own folder.
    pub fn choices_for(&self, number: usize) -> Vec<Choice> {
        let mut choices: Vec<Choice> = self
            .existing
            .iter()
            .chain(&self.proposed_new)
            .cloned()
            .collect();
        let folder = self
            .rows
            .iter()
            .find(|row| row.session.number == number)
            .map(|row| row.session.folder.clone())
            .filter(|folder| !folder.is_empty());
        if let Some(folder) = folder {
            let own = Destination::New {
                root_dir: folder.clone(),
            };
            if !choices.iter().any(|choice| choice.destination == own) {
                choices.push(Choice {
                    name: folder_name(&folder),
                    is_new: true,
                    destination: own,
                });
            }
        }
        choices
    }

    /// How a destination reads in the selector.
    pub fn choice_of(&self, destination: &Destination) -> Option<Choice> {
        let listed = self.existing.iter().chain(&self.proposed_new);
        if let Some(choice) = listed.clone().find(|c| &c.destination == destination) {
            return Some(choice.clone());
        }
        match destination {
            Destination::New { root_dir } => Some(Choice {
                name: folder_name(root_dir),
                is_new: true,
                destination: destination.clone(),
            }),
            _ => None,
        }
    }

    pub fn marked(&self) -> usize {
        self.rows.iter().filter(|row| row.marked).count()
    }

    /// What to import, or the numbers of the marked sessions that still
    /// have no category.
    pub fn chosen(&self) -> Result<Vec<Chosen>, Vec<usize>> {
        let marked = self.rows.iter().filter(|row| row.marked);
        let without: Vec<usize> = marked
            .clone()
            .filter(|row| row.destination.is_none())
            .map(|row| row.session.number)
            .collect();
        if !without.is_empty() {
            return Err(without);
        }
        Ok(marked
            .filter_map(|row| Some(row.session.chosen(row.destination.clone()?)))
            .collect())
    }
}
