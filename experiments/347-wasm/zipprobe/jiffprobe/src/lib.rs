#[unsafe(no_mangle)]
pub extern "C" fn probe() -> usize {
    jiff::Zoned::now().strftime("%Y-%m-%d").to_string().len()
}
