pub fn value(value: Option<&str>) {
    match value {
        Some(value) => println!("{value}"),
        None => println!("(not set)"),
    }
}

pub fn changed(key: &str, value: Option<&str>) {
    match value {
        Some(value) => println!("{key} = {value}"),
        None => println!("{key} unset"),
    }
}
