use anyhow::{Context, bail};
use lithemux_core::modules::category::{
    BoardStore, FsDirectories, NewCategory, Slug, create_category, get_category, list_categories,
    locate_directory, remove_category,
};
use lithemux_core::modules::launch::ensure_closed;
use lithemux_core::modules::session::list_sessions;
use lithemux_core::platform::clock::system_clock::now_unix;

use crate::args::CatAddArgs;
use crate::output;
use crate::prompt;
use crate::wiring::Wiring;

pub fn add(args: CatAddArgs) -> anyhow::Result<()> {
    let wiring = Wiring::open()?;
    let boards = wiring.boards();
    let category = create_category(
        &wiring.categories(),
        &boards,
        NewCategory {
            slug: Slug::parse(&args.slug)?,
            name: args.name,
            root_dir: locate_directory(&FsDirectories, &args.root)?,
            color: args.color,
        },
        now_unix(),
    )?;
    output::category::created(&category, &boards.path(&category.slug));
    Ok(())
}

pub fn list(json: bool) -> anyhow::Result<()> {
    let wiring = Wiring::open()?;
    let categories = list_categories(&wiring.categories())?;
    if json {
        output::category::list_json(&categories)
    } else {
        output::category::list(&categories);
        Ok(())
    }
}

pub fn remove(slug: &str, yes: bool) -> anyhow::Result<()> {
    let wiring = Wiring::open()?;
    let category = get_category(&wiring.categories(), slug)?;
    for session in list_sessions(&wiring.sessions(), &category)? {
        ensure_closed(&wiring.locks(), &session.id)
            .with_context(|| format!("{}/{}", category.slug, session.slug))?;
    }
    let question = format!(
        "Remove category \"{}\" and all its sessions?",
        category.slug
    );
    if !yes && !prompt::confirm(&question)? {
        bail!("not removed (pass --yes to confirm)");
    }
    remove_category(&wiring.categories(), slug)?;
    output::category::removed(&category, &wiring.boards().path(&category.slug));
    Ok(())
}
