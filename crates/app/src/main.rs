mod text_check;

use std::env;
use std::process::ExitCode;

use lithemux_app::app_shell::run;
use lithemux_app::launch_arguments::LaunchRequest;

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if arguments.first().map(String::as_str) == Some(text_check::FLAG) {
        return text_check::run();
    }
    match LaunchRequest::parse(&arguments) {
        Ok(request) => run(request),
        Err(problem) => {
            eprintln!("lithemux-app: {problem}");
            ExitCode::from(2)
        }
    }
}
