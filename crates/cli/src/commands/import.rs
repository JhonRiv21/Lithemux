use anyhow::Context;
use lithemux_core::modules::category::get_category;
use lithemux_core::modules::import::{
    Choices, CmuxSessionFile, Destination, ImportPorts, SqliteImportStore, apply_import, choose,
    propose_import,
};
use lithemux_core::modules::launch::ClaudeProjectsTranscriptLocator;
use lithemux_core::platform::clock::system_clock::now_unix;

use crate::args::ImportCmuxArgs;
use crate::output;
use crate::output::import::Shown;
use crate::wiring::Wiring;

pub fn cmux(args: ImportCmuxArgs) -> anyhow::Result<()> {
    let wiring = Wiring::open()?;
    let user_home = std::env::home_dir().context("the home folder of the user is unknown")?;
    let cmux = match &args.from {
        Some(path) => CmuxSessionFile::at(path),
        None => CmuxSessionFile::of_user(&user_home),
    };
    let store = SqliteImportStore::new(&wiring.db);
    let ports = ImportPorts {
        cmux: &cmux,
        categories: &wiring.categories(),
        sessions: &wiring.sessions(),
        transcripts: &ClaudeProjectsTranscriptLocator::from_env(),
        boards: &wiring.boards(),
        store: &store,
    };
    let home = user_home.to_string_lossy().into_owned();
    let proposal = propose_import(&ports, Some(&home))?;
    let into = match &args.into {
        Some(slug) => Some(Destination::Existing {
            category_id: get_category(&wiring.categories(), slug)?.id,
        }),
        None => None,
    };
    let choices = Choices {
        skip: &args.skip,
        include: &args.include,
        into: into.as_ref(),
    };
    let categories = lithemux_core::modules::category::list_categories(&wiring.categories())?;
    let shown = Shown {
        proposal: &proposal,
        choices: &choices,
        categories: &categories,
        home: &home,
    };
    if !args.apply {
        return if args.json {
            output::import::proposal_json(&shown)
        } else {
            output::import::proposal(&shown);
            Ok(())
        };
    }
    let chosen = choose(&proposal, &choices)?;
    let imported = apply_import(&ports, &chosen, now_unix())?;
    output::import::imported(&imported);
    Ok(())
}
