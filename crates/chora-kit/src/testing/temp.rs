//! Tests of the counted temporary folder of the tests.

use super::TempDir;

/// Two folders of one name in one process never clash, and each goes
/// when its owner goes.
#[test]
fn same_name_twice_gives_two_folders_removed_on_drop() {
    let first = TempDir::new("chora-temp-twice");
    let second = TempDir::new("chora-temp-twice");
    assert_ne!(first.path(), second.path());
    assert!(first.path().is_dir() && second.path().is_dir());
    let (a, b) = (first.path().to_path_buf(), second.path().to_path_buf());
    std::fs::write(a.join("file"), b"x").unwrap();
    drop(first);
    assert!(!a.exists(), "the first folder is removed on drop");
    assert!(b.is_dir(), "the second folder stays while it lives");
    drop(second);
    assert!(!b.exists(), "the second folder is removed on drop");
}
