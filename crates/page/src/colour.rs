pub(crate) fn colour([red, green, blue]: [u8; 3]) -> String {
    format!("rgb({red} {green} {blue})")
}
