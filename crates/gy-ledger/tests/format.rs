use gy_ledger::{FormatVersion, format, location};

#[test]
fn a_missing_format_file_is_an_error() {
    let temp = tempfile::tempdir().unwrap();
    assert!(format::read(temp.path()).is_err());
}

#[test]
fn a_newer_format_is_rejected() {
    let temp = tempfile::tempdir().unwrap();
    format::write(temp.path(), FormatVersion(FormatVersion::CURRENT.0 + 1)).unwrap();
    assert!(format::read(temp.path()).is_err());
}

#[test]
fn the_current_format_opens() {
    let temp = tempfile::tempdir().unwrap();
    format::write(temp.path(), FormatVersion::CURRENT).unwrap();
    assert_eq!(format::read(temp.path()).unwrap(), FormatVersion::CURRENT);
}

#[test]
fn an_older_format_reaches_the_migration_frame() {
    let temp = tempfile::tempdir().unwrap();
    let old = FormatVersion(0);
    format::write(temp.path(), old).unwrap();
    assert_eq!(format::read(temp.path()).unwrap(), old);
    assert!(format::migrate(old, FormatVersion::CURRENT).is_err());
    assert!(format::migrate(FormatVersion::CURRENT, FormatVersion::CURRENT).is_ok());
}

#[test]
fn a_corrupt_format_file_is_an_error() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join(format::FILE), "not a version\n").unwrap();
    assert!(format::read(temp.path()).is_err());
}

#[test]
fn the_location_key_is_stable_and_root_specific() {
    let temp = tempfile::tempdir().unwrap();
    let one = temp.path().join("one");
    let two = temp.path().join("two");
    std::fs::create_dir_all(&one).unwrap();
    std::fs::create_dir_all(&two).unwrap();
    assert_eq!(location::key(&one), location::key(&one));
    assert_ne!(location::key(&one), location::key(&two));
    assert_eq!(
        location::dir_in(temp.path(), &one),
        temp.path().join("gy").join(location::key(&one))
    );
    assert_eq!(location::key(&one).len(), 8);
}
