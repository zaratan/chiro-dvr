#[cfg(unix)]
use std::os::fd::AsFd;

#[cfg(all(test, unix))]
mod tests;

#[cfg(unix)]
pub(crate) fn columns(fd: impl AsFd) -> Option<usize> {
    let size = rustix::termios::tcgetwinsize(fd).ok()?;
    (size.ws_col > 0).then_some(usize::from(size.ws_col))
}

#[cfg(unix)]
pub(crate) fn stderr_columns() -> Option<usize> {
    columns(std::io::stderr())
}

#[cfg(not(unix))]
pub(crate) const fn stderr_columns() -> Option<usize> {
    None
}
