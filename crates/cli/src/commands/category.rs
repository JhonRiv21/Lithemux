use std::path::Path;

use anyhow::{Context, bail};
use lithemux_core::modules::category::{
    BoardStore, NewCategory, Slug, create_category, get_category, list_categories, remove_category,
};
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
            root_dir: existing_dir(&args.root)?,
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

pub fn existing_dir(path: &Path) -> anyhow::Result<String> {
    let absolute = path
        .canonicalize()
        .with_context(|| format!("{} is not an existing directory", path.display()))?;
    if !absolute.is_dir() {
        bail!("{} is not a directory", absolute.display());
    }
    Ok(absolute.display().to_string())
}
