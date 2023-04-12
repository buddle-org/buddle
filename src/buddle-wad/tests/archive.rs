use std::sync::Arc;

use buddle_wad::{Archive, Interner};

#[test]
fn open_mmap() {
    let _ = Archive::mmap("tests/data/Test.wad", true).unwrap();
}

#[test]
fn open_heap() {
    let _ = Archive::heap("tests/data/Test.wad", true).unwrap();
}

#[test]
fn uncompressed() {
    let archive = Archive::heap("tests/data/Test.wad", true).unwrap();
    let mut interner = Interner::new(&archive);

    // Extract the raw file contents which should be uncompressed.
    let (compressed, data) = archive.file_raw("uncompressed.mp3").unwrap();
    assert!(!compressed);

    // Retrieve the file again through the interner.
    let handle = interner.intern("uncompressed.mp3").unwrap();
    let data2 = interner.fetch(handle).unwrap();

    // The data should be the same.
    assert_eq!(data, data2);
}

#[test]
fn subdir() {
    let archive = Archive::heap("tests/data/Test.wad", true).unwrap();
    let mut interner = Interner::new(&archive);

    let handle = interner.intern("subdir/subdir_text1.txt").unwrap();
    let data = interner.fetch(handle).unwrap();

    assert_eq!(data, b"this is subdir text1\n");
}

#[test]
fn two_files() {
    let archive = Archive::heap("tests/data/Test.wad", true).unwrap();
    let mut interner = Interner::new(&archive);

    let text1 = interner.intern("text1.txt").unwrap();
    let subdir = interner.intern("subdir/subdir_text1.txt").unwrap();

    let subdir = interner.fetch(subdir).unwrap();
    let text1 = interner.fetch(text1).unwrap();

    assert_ne!(text1, subdir);
}

#[test]
fn intern_twice() {
    let archive = Archive::heap("tests/data/Test.wad", true).unwrap();
    let mut interner = Interner::new(&archive);

    let handle = interner.intern("text1.txt").unwrap();
    let handle2 = interner.intern("text1.txt").unwrap();

    assert_eq!(
        interner.fetch(handle).unwrap(),
        interner.fetch(handle2).unwrap()
    );
}

#[test]
fn invalidate() {
    let archive = Archive::heap("tests/data/Test.wad", true).unwrap();
    let mut interner = Interner::new(&archive);

    // Intern a file and validate its contents.
    let text1 = interner.intern("text1.txt").unwrap();
    assert_eq!(interner.fetch(text1).unwrap(), b"this is text1\n");

    // Then invalidate the entire interner state.
    interner.invalidate_all();

    // Now we shouldn't be able to fetch the file anymore.
    assert_eq!(interner.fetch(text1), None);

    // And even if we intern it again...
    let text1_new = interner.intern("text1.txt").unwrap();

    // ...the old one should not produce any matches.
    assert_ne!(text1, text1_new);
    assert_eq!(interner.fetch(text1), None);
    assert_eq!(interner.fetch(text1_new).unwrap(), b"this is text1\n");
}

#[test]
fn arc_interner() {
    let archive = Archive::heap("tests/data/Test.wad", true)
        .map(Arc::new)
        .unwrap();
    let mut interner = Interner::new(archive.clone());

    let handle = interner.intern("subdir/subdir_text1.txt").unwrap();
    let data = interner.fetch(handle).unwrap();

    assert_eq!(data, b"this is subdir text1\n");
}
