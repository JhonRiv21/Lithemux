/// Temporary (PLAN §2.1): opens a session with the session host.
pub const OPEN_FLAG: &str = "--open";
/// Runs the sessions without a window. For the tests of the app's exit.
pub const NO_WINDOW_FLAG: &str = "--no-window";

/// `lithemux-app --open <category>/<session> [--open …] [--no-window]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenRequest {
    /// In the order given, without repeats.
    pub targets: Vec<String>,
    pub with_window: bool,
}

impl OpenRequest {
    /// `None` when the arguments do not ask to open sessions at all;
    /// otherwise the request, or what is wrong with it.
    pub fn parse(arguments: &[String]) -> Option<Result<Self, String>> {
        if !arguments.iter().any(|argument| argument == OPEN_FLAG) {
            return None;
        }
        Some(Self::read(arguments))
    }

    fn read(arguments: &[String]) -> Result<Self, String> {
        let mut request = OpenRequest {
            targets: Vec::new(),
            with_window: true,
        };
        let mut rest = arguments.iter();
        while let Some(argument) = rest.next() {
            match argument.as_str() {
                NO_WINDOW_FLAG => request.with_window = false,
                OPEN_FLAG => {
                    let target = rest
                        .next()
                        .filter(|value| !value.starts_with("--"))
                        .ok_or_else(|| format!("{OPEN_FLAG} needs a <category>/<session>"))?;
                    // Two views over one terminal are not supported: a
                    // session has a single channel of repaint requests.
                    if request.targets.contains(target) {
                        return Err(format!("{target} was given twice"));
                    }
                    request.targets.push(target.clone());
                }
                other => {
                    return Err(format!(
                        "unexpected argument `{other}`; use {OPEN_FLAG} <category>/<session>"
                    ));
                }
            }
        }
        Ok(request)
    }
}
