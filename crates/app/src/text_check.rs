//! `lithemux-app --check-text`: says whether this build can draw text, with
//! no window. Exits non-zero when it cannot.

use std::process::ExitCode;

use lithemux_app::terminal_view::probe_text_rendering;

pub const FLAG: &str = "--check-text";

pub fn run() -> ExitCode {
    let platform = gpui_platform::current_platform(true);
    match probe_text_rendering(platform.text_system().as_ref()) {
        Ok(()) => {
            println!("text rendering: ok");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("lithemux: {error}");
            ExitCode::FAILURE
        }
    }
}
