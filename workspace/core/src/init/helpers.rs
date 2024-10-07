pub(crate) fn validate_name(name: &String) -> bool {
    name.len() > 0 && name.len() <= 64
}
