use std::path::Path;

use lithemux_core::modules::category::Category;

pub fn created(category: &Category, board: &Path) {
    println!("Created category \"{}\" ({})", category.slug, category.name);
    println!("  root:  {}", category.root_dir);
    println!("  board: {}", board.display());
}

pub fn removed(category: &Category, board: &Path) {
    println!("Removed category \"{}\" and its sessions.", category.slug);
    println!("The board is still on disk: {}", board.display());
}

pub fn list(categories: &[Category]) {
    if categories.is_empty() {
        println!("No categories yet. Create one with `lithemux cat add`.");
        return;
    }
    let width = categories.iter().map(|c| c.slug.len()).max().unwrap_or(0);
    for category in categories {
        println!(
            "{:width$}  {}  {}",
            category.slug, category.name, category.root_dir
        );
    }
}

pub fn list_json(categories: &[Category]) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(categories)?);
    Ok(())
}
