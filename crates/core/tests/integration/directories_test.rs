use std::fs;
use std::path::Path;

use lithemux_core::modules::category::{CategoryError, FsDirectories, locate_directory};

// PLAN §0.6 `cat add --root` / `new --cwd`: stored as an absolute path
#[test]
fn a_directory_is_stored_in_its_absolute_form() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("repo/src")).unwrap();
    let roundabout = temp.path().join("repo/src/..");

    let located = locate_directory(&FsDirectories, &roundabout).unwrap();

    let expected = temp.path().join("repo").canonicalize().unwrap();
    assert_eq!(Path::new(&located), expected);
}

// PLAN §0.6: a category cannot be rooted in something that is not a directory
#[test]
fn a_missing_path_and_a_file_are_not_directories() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("notes.txt");
    fs::write(&file, "x").unwrap();

    for path in [temp.path().join("nowhere"), file] {
        let error = locate_directory(&FsDirectories, &path).unwrap_err();
        assert!(matches!(error, CategoryError::NotADirectory(_)));
        assert!(error.to_string().contains("is not an existing directory"));
    }
}
