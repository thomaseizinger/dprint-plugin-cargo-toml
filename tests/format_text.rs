use dprint_cargo_toml::{PluginHandler, Configuration};
use std::path::Path;
use dprint_core::plugins::PluginHandler as _;

#[test]
fn formats_cnd_cargo_toml_correctly() {
    let input = include_str!("./testcase_cnd/Cargo.toml");
    let expected = include_str!("./testcase_cnd/Cargo.expected.toml");

    let output = PluginHandler.format_text(
        Path::new("/testcase_cnd/Cargo.toml"),
        input,
        &Configuration {},
        |_, _, _| unimplemented!()
    )
    .unwrap();

    assert_eq!(output, expected);
}

#[test]
fn formats_comit_cargo_toml_correctly() {
    let input = include_str!("./testcase_comit/Cargo.toml");
    let expected = include_str!("./testcase_comit/Cargo.expected.toml");

    let output = PluginHandler.format_text(
        Path::new("/testcase_comit/Cargo.toml"),
        input,
        &Configuration {},
        |_, _, _| unimplemented!()
    )
    .unwrap();

    assert_eq!(output, expected);
}
