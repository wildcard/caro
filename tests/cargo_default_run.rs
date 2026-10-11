//! Regression guard for #1222 / #1164: the package has two [[bin]] targets
//! (caro, generate-schema), so plain `cargo run` (README, the CLI test
//! runners' fallback) only works while `default-run = "caro"` is set.

#[test]
fn cargo_run_defaults_to_caro_binary() {
    let manifest: toml::Value =
        toml::from_str(include_str!("../Cargo.toml")).expect("Cargo.toml parses");
    let package = &manifest["package"];
    assert_eq!(
        package.get("default-run").and_then(|v| v.as_str()),
        Some("caro"),
        "Cargo.toml [package] must set default-run = \"caro\" so `cargo run` is unambiguous"
    );

    let bins: Vec<&str> = manifest["bin"]
        .as_array()
        .expect("[[bin]] targets")
        .iter()
        .filter_map(|b| b.get("name").and_then(|n| n.as_str()))
        .collect();
    assert!(
        bins.contains(&"caro"),
        "default-run must name a real [[bin]]: {bins:?}"
    );
}
