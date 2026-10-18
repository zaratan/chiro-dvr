#[unsafe(no_mangle)]
pub extern "C" fn probe(size: u64) -> u64 {
    let block = vec![0x5a_u8; size as usize];
    block.iter().map(|&b| u64::from(b)).sum()
}
