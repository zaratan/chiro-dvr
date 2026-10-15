use std::io::ErrorKind;

use super::Access;

#[test]
fn each_kind_of_system_refusal_has_its_own_reason_and_any_other_kind_reads_as_unavailable() {
    let read: Vec<Access> = [
        ErrorKind::PermissionDenied,
        ErrorKind::NotADirectory,
        ErrorKind::NotFound,
        ErrorKind::ReadOnlyFilesystem,
        ErrorKind::StorageFull,
        ErrorKind::QuotaExceeded,
        ErrorKind::Interrupted,
    ]
    .into_iter()
    .map(Access::from)
    .collect();
    assert_eq!(
        read,
        [
            Access::Refused,
            Access::NotAFolder,
            Access::Missing,
            Access::ReadOnly,
            Access::DiskFull,
            Access::DiskFull,
            Access::Unavailable,
        ]
    );
}
