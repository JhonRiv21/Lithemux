#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Ok,
    Warn,
    Fail,
}

pub struct Check {
    pub name: &'static str,
    pub level: Level,
    pub detail: String,
}

impl Check {
    pub fn new(name: &'static str, level: Level, detail: String) -> Self {
        Check {
            name,
            level,
            detail,
        }
    }
}

pub fn report(checks: &[Check]) {
    let width = checks.iter().map(|c| c.name.len()).max().unwrap_or(0);
    for check in checks {
        let label = match check.level {
            Level::Ok => "ok  ",
            Level::Warn => "warn",
            Level::Fail => "FAIL",
        };
        println!("{label}  {:width$}  {}", check.name, check.detail);
    }
}
