mod hosted_sessions;
mod hosted_window;
mod quit_signals;
mod spike_window;
mod text_check;

use std::env;
use std::process::ExitCode;

use lithemux_app::launch_arguments::OpenRequest;

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if arguments.first().map(String::as_str) == Some(text_check::FLAG) {
        return text_check::run();
    }
    match OpenRequest::parse(&arguments) {
        Some(Ok(request)) => hosted_sessions::run(request),
        Some(Err(problem)) => {
            eprintln!("lithemux-app: {problem}");
            ExitCode::from(2)
        }
        None => {
            spike_window::run();
            ExitCode::SUCCESS
        }
    }
}
