#![cfg(test)]
use tempfile::tempdir;
use form::create_directory_structure;
use std::fs::File;
use std::path::Path;
use std::io::Read;
use syn::parse_file;

#[test]
fn test_from_reference_files() {

    let before_file = std::str::from_utf8(include_bytes!("resources/small-lib-before.rs")).unwrap();

    let expected_lib = include_bytes!("resources/after/lib.rs");
    let expected_interrupt = include_bytes!("resources/after/interrupt.rs");
    let expected_ac = include_bytes!("resources/after/ac.rs");
    let expected_ac2 = include_bytes!("resources/after/ac/ac2.rs");
    let expected_ac3 = include_bytes!("resources/after/ac/ac2/ac3.rs");

    let lib_dir = tempdir().unwrap();
    create_directory_structure(lib_dir.path(), before_file.to_string()).unwrap();

    compare_to_expected(expected_lib, lib_dir.path().join("lib.rs"));
    compare_to_expected(expected_interrupt, lib_dir.path().join("interrupt.rs"));
    compare_to_expected(expected_ac, lib_dir.path().join("ac.rs"));
    compare_to_expected(expected_ac2, lib_dir.path().join("ac/ac2.rs"));
    compare_to_expected(expected_ac3, lib_dir.path().join("ac/ac2/ac3.rs"));
}

fn compare_to_expected<P: AsRef<Path>>(expected: &[u8], path: P) {
    let expected = parse_file(std::str::from_utf8(expected).unwrap()).unwrap();

    let mut found_string = String::new();
    let mut found_file = File::open(path).unwrap();
    found_file.read_to_string(&mut found_string).unwrap();
    let found = parse_file(&found_string).unwrap();
    assert_eq!(expected, found)
}
