use buddle_wad::{self, InternedArchive};

#[test]
fn test_read_data() {
    let archive = buddle_wad::Archive::heap("tests/data/Test.wad", false).unwrap();
    let mut word_archive = InternedArchive::new(archive);

    let data = word_archive.get("text1.txt").unwrap();

    assert_eq!(data, b"this is text1\n")
}
