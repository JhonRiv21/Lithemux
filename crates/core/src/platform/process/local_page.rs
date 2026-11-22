//! Shows `http://localhost:<port>` in the user's default browser. The app
//! itself opens no connection: it hands the address to `/usr/bin/open`.

use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::process::Stdio;

use crate::platform::process::session_markers::command_without_session_markers;

const OPENER: &str = "/usr/bin/open";

pub fn local_page_url(port: u16) -> String {
    format!("http://localhost:{port}")
}

pub fn open_local_page(port: u16) -> io::Result<()> {
    hand_to_the_system(local_page_url(port).as_ref())
}

/// Shows a folder in the Finder.
pub fn open_folder(folder: &Path) -> io::Result<()> {
    hand_to_the_system(folder.as_os_str())
}

fn hand_to_the_system(what: &OsStr) -> io::Result<()> {
    let mut opener = command_without_session_markers(OPENER);
    opener
        .arg(what)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    // `open` returns as soon as it has handed the address over; it is
    // waited for on a thread so that it leaves no zombie.
    let mut child = opener.spawn()?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}
