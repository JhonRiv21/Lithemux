//! The parts of cmux's `session-com.cmuxterm.app.json` (version 1) that the
//! import uses. Fields not declared here, the notifications with their text
//! among them, are skipped by the parser and never kept.

use serde::Deserialize;

use crate::modules::import::domain::cmux_panel::CmuxPanel;

/// Read first, alone: what the rest means depends on it.
#[derive(Deserialize)]
pub struct Versioned {
    pub version: Option<serde_json::Value>,
}

/// A list may also come as `null`: it is read as empty.
#[derive(Deserialize)]
pub struct SessionsFile {
    windows: Option<Vec<Window>>,
}

#[derive(Deserialize)]
struct Window {
    #[serde(rename = "tabManager")]
    tab_manager: Option<TabManager>,
}

#[derive(Deserialize)]
struct TabManager {
    workspaces: Option<Vec<Workspace>>,
}

#[derive(Deserialize)]
struct Workspace {
    #[serde(rename = "customTitle")]
    custom_title: Option<String>,
    #[serde(rename = "processTitle")]
    process_title: Option<String>,
    #[serde(rename = "currentDirectory")]
    current_directory: Option<String>,
    #[serde(rename = "isPinned")]
    is_pinned: Option<bool>,
    panels: Option<Vec<Panel>>,
}

#[derive(Deserialize)]
struct Panel {
    directory: Option<String>,
    terminal: Option<Terminal>,
}

#[derive(Deserialize)]
struct Terminal {
    agent: Option<Agent>,
    #[serde(rename = "workingDirectory")]
    working_directory: Option<String>,
}

#[derive(Deserialize)]
struct Agent {
    kind: Option<String>,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
    #[serde(rename = "workingDirectory")]
    working_directory: Option<String>,
}

impl SessionsFile {
    /// Every panel of every workspace, in the order of the file. The folder
    /// is the most precise one given: the agent's, the terminal's, the
    /// panel's, the workspace's.
    pub fn panels(self) -> Vec<CmuxPanel> {
        let workspaces = self
            .windows
            .unwrap_or_default()
            .into_iter()
            .filter_map(|window| window.tab_manager)
            .flat_map(|manager| manager.workspaces.unwrap_or_default());
        let mut panels = Vec::new();
        for workspace in workspaces {
            for panel in workspace.panels.unwrap_or_default() {
                let terminal = panel.terminal;
                let agent = terminal.as_ref().and_then(|t| t.agent.as_ref());
                let folder = agent
                    .and_then(|a| a.working_directory.clone())
                    .or_else(|| terminal.as_ref().and_then(|t| t.working_directory.clone()))
                    .or(panel.directory)
                    .or_else(|| workspace.current_directory.clone());
                panels.push(CmuxPanel {
                    custom_title: workspace.custom_title.clone(),
                    process_title: workspace.process_title.clone(),
                    folder,
                    pinned: workspace.is_pinned.unwrap_or(false),
                    agent: agent.and_then(|a| a.kind.clone()),
                    claude_session_id: agent.and_then(|a| a.session_id.clone()),
                });
            }
        }
        panels
    }
}
