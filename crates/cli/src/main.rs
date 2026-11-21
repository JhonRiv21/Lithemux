mod args;
mod commands;
mod output;
mod prompt;
mod wiring;

use std::process::ExitCode;

use clap::Parser;

use crate::args::{CatCommand, Cli, Command, ConfigCommand};

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("lithemux: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> anyhow::Result<u8> {
    match cli.command {
        Command::Cat { command } => match command {
            CatCommand::Add(args) => commands::category::add(args)?,
            CatCommand::List { json } => commands::category::list(json)?,
            CatCommand::Rm { slug, yes } => commands::category::remove(&slug, yes)?,
        },
        Command::New(args) => commands::session::new(args)?,
        Command::Set(args) => commands::session::set(args)?,
        Command::Rm { target, yes } => commands::session::remove(&target, yes)?,
        Command::Context { target } => commands::session::context(&target)?,
        Command::Status { category, json } => commands::status::show(category.as_deref(), json)?,
        Command::Config { command } => match command {
            ConfigCommand::Get { key } => commands::config::get(&key)?,
            ConfigCommand::Set { key, value } => commands::config::set(&key, &value)?,
            ConfigCommand::Unset { key } => commands::config::unset(&key)?,
        },
        Command::Open { target } => return commands::launch::open(&target),
        Command::Doctor => return commands::doctor::run(),
    }
    Ok(0)
}
