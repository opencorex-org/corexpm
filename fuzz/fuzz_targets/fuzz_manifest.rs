//! Fuzz target for package.json manifest parser.

use corex_manifest::PackageManifest;

fn main() {
    let input = r#"{"name": "@corex/fuzz-test", "dependencies": {"react": "^18.0.0"}}"#;
    let _ = PackageManifest::parse_json(input);
}
