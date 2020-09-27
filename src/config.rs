fn get_prop_str(section: &str, property: &str) -> Option<String> {
    std::env::var(format!(
        "{}__{}",
        section.to_uppercase(),
        property.to_uppercase()
    ))
    .ok()
}

pub fn get_prop<T: std::str::FromStr>(section: &str, property: &str) -> Option<T> {
    let val = get_prop_str(section, property).map(|x| x.parse::<T>());
    match val {
        Some(Ok(val)) => Some(val),
        _ => None,
    }
}
