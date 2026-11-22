/// Opens a session with the session host. Only without a window: with one,
/// sessions are opened from the sidebar.
pub const OPEN_FLAG: &str = "--open";
/// Runs the session host without any interface. For the tests of how the
/// app exits.
pub const NO_WINDOW_FLAG: &str = "--no-window";

/// `lithemux-app`, or `lithemux-app --no-window [--open <category>/<session>]…`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchRequest {
    pub with_window: bool,
    /// Sessions to open without a window, in the order given, without
    /// repeats.
    pub targets: Vec<String>,
}

impl LaunchRequest {
    /// What the arguments ask for, or what is wrong with them.
    pub fn parse(arguments: &[String]) -> Result<Self, String> {
        let mut request = LaunchRequest {
            with_window: true,
            targets: Vec::new(),
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
                    if request.targets.contains(target) {
                        return Err(format!("{target} was given twice"));
                    }
                    request.targets.push(target.clone());
                }
                other => return Err(format!("unexpected argument `{other}`")),
            }
        }
        if request.with_window && !request.targets.is_empty() {
            return Err(format!(
                "{OPEN_FLAG} needs {NO_WINDOW_FLAG}: in the window, sessions are opened from the sidebar"
            ));
        }
        Ok(request)
    }
}
