#[cfg(test)]
mod tests;

#[must_use]
pub(crate) fn copy_rows(
    source: &[u8],
    source_stride: usize,
    destination: &mut [u8],
    destination_stride: usize,
    row: usize,
    rows: usize,
) -> bool {
    if source_stride == 0 || destination_stride == 0 {
        return rows == 0;
    }
    let copied = destination
        .chunks_mut(destination_stride)
        .zip(source.chunks(source_stride))
        .take(rows)
        .try_fold(0, |count, (to, from)| {
            to.get_mut(..row)?.copy_from_slice(from.get(..row)?);
            Some(count + 1)
        });
    copied == Some(rows)
}
