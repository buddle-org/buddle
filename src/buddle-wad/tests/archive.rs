use buddle_wad::{Archive, Interner};

#[test]
fn mmap() {
    let _ = Archive::mmap("tests/data/Test.wad", true).unwrap();
}

#[test]
fn heap() {
    let _ = Archive::heap("tests/data/Test.wad", true).unwrap();
}

#[test]
fn uncompressed() {
    let archive = Archive::heap("tests/data/Test.wad", true).unwrap();
    let mut interner = Interner::new();

    // Extract the raw file contents which should be uncompressed.
    let (_, data) = archive.file_raw("uncompressed.mp3").unwrap();

    // Retrieve the file again through the interner (which should decompress).
    let handle = interner.intern(&archive, "uncompressed.mp3").unwrap();
    let data2 = interner.fetch(handle).unwrap();

    // But since the data is not compressed to begin with, these should be the same.
    assert_eq!(data, data2);
}

#[test]
fn subdir() {
    let archive = Archive::heap("tests/data/Test.wad", true).unwrap();
    let mut interner = Interner::new();

    let handle = interner
        .intern(&archive, "subdir/subdir_text1.txt")
        .unwrap();

    let data = interner.fetch(handle).unwrap();

    assert_eq!(data, b"this is subdir text1\n");
}

#[test]
fn two_files() {
    let archive = Archive::heap("tests/data/Test.wad", true).unwrap();
    let mut interner = Interner::new();

    // We need to intern the files first before borrowing them.
    let text1 = interner.intern(&archive, "text1.txt").unwrap();

    let subdir = interner
        .intern(&archive, "subdir/subdir_text1.txt")
        .unwrap();

    // And now we can simultaneously access them.
    let text1 = interner.fetch(text1).unwrap();
    let subdir = interner.fetch(subdir).unwrap();

    assert_ne!(text1, subdir);
}

#[test]
fn intern_twice() {
    let archive = Archive::heap("tests/data/Test.wad", true).unwrap();
    let mut interner = Interner::new();

    let handle = interner.intern(&archive, "text1.txt").unwrap();
    let handle2 = interner.intern(&archive, "text1.txt").unwrap();

    assert_eq!(
        interner.fetch(handle).unwrap(),
        interner.fetch(handle2).unwrap()
    );
}

#[test]
fn invalidate() {
    let archive = Archive::heap("tests/data/Test.wad", true).unwrap();
    let mut interner = Interner::new();

    // Intern a file and validate its contents.
    let text1 = interner.intern(&archive, "text1.txt").unwrap();
    assert_eq!(interner.fetch(text1).unwrap(), b"this is text1\n");

    // Then invalidate the entire interner state.
    interner.invalidate_all();

    // Now we shouldn't be able to fetch the file anymore.
    assert_eq!(interner.fetch(text1), None);
}
