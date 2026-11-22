use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "lithemux",
    version,
    about = "Lightweight manager for terminal and Claude Code sessions"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(Subcommand)]
pub enum CliCommand {
    /// Manage categories
    Cat {
        #[command(subcommand)]
        command: CatCommand,
    },
    /// Create a session: <category>/<session>
    New(NewArgs),
    /// Open a session in this terminal
    Open { target: String },
    /// Show categories and sessions with their current state
    Status {
        category: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Print the context a session would receive at startup
    Context { target: String },
    /// Change a session
    Set(SetArgs),
    /// Remove a session (the Claude transcript is kept)
    Rm {
        target: String,
        #[arg(long)]
        yes: bool,
    },
    /// Read or change a setting (claude_path, context_language)
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Check the installation
    Doctor,
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    Get { key: String },
    Set { key: String, value: String },
    Unset { key: String },
}

#[derive(Subcommand)]
pub enum CatCommand {
    /// Create a category
    Add(CatAddArgs),
    /// List categories
    List {
        #[arg(long)]
        json: bool,
    },
    /// Remove a category and its sessions (the board file is kept)
    Rm {
        slug: String,
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Args)]
pub struct CatAddArgs {
    pub slug: String,
    #[arg(long)]
    pub name: String,
    /// Default working directory of the category's sessions
    #[arg(long)]
    pub root: PathBuf,
    #[arg(long)]
    pub color: Option<String>,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum ProfileArg {
    Claude,
    Shell,
}

#[derive(Args)]
pub struct NewArgs {
    pub target: String,
    /// What this session is for
    #[arg(long)]
    pub role: String,
    /// Make it the mother of the category
    #[arg(long)]
    pub mother: bool,
    #[arg(long, value_enum, default_value = "claude")]
    pub profile: ProfileArg,
    /// Working directory, when different from the category root
    #[arg(long)]
    pub cwd: Option<PathBuf>,
    #[arg(long)]
    pub model: Option<String>,
}

#[derive(Args)]
pub struct SetArgs {
    pub target: String,
    #[arg(long)]
    pub role: Option<String>,
    /// Make it the mother (`--mother false` turns it back into a child)
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    pub mother: Option<bool>,
    /// Never hibernate this session
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    pub keep_alive: Option<bool>,
    #[arg(long, conflicts_with = "default_model")]
    pub model: Option<String>,
    /// Go back to the user's default model
    #[arg(long)]
    pub default_model: bool,
}
