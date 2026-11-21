pub fn since(now: i64, at: Option<i64>) -> String {
    let Some(at) = at else {
        return "never".to_string();
    };
    let seconds = (now - at).max(0);
    let (minutes, hours, days) = (seconds / 60, seconds / 3600, seconds / 86_400);
    if minutes < 1 {
        "just now".to_string()
    } else if hours < 1 {
        format!("{minutes} min ago")
    } else if days < 1 {
        format!("{hours} h ago")
    } else if days == 1 {
        "1 day ago".to_string()
    } else {
        format!("{days} days ago")
    }
}
