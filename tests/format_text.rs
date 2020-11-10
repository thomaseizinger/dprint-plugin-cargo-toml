use dprint_cargo_toml::{format_text, Configuration};
use std::path::Path;

#[test]
fn formats_cnd_cargo_toml_correctly() {
    let input = include_str!("./testcase_cnd/Cargo.toml");
    let expected = include_str!("./testcase_cnd/Cargo.expected.toml");

    let output = format_text(
        Path::new("/testcase_cnd/Cargo.toml"),
        input,
        &Configuration {},
    )
    .unwrap();

    assert_eq!(output, expected);
}

#[test]
fn formats_comit_cargo_toml_correctly() {
    let input = include_str!("./testcase_comit/Cargo.toml");
    let expected = include_str!("./testcase_comit/Cargo.expected.toml");

    let output = format_text(
        Path::new("/testcase_comit/Cargo.toml"),
        input,
        &Configuration {},
    )
    .unwrap();

    assert_eq!(output, expected);
}
