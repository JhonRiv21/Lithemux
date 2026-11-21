use std::fs;

pub fn board_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}
