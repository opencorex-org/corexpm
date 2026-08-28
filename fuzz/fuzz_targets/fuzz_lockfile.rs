//! Fuzz target for lockfile canonical JSON parser and validator.

use corex_lockfile::Lockfile;

fn main() {
    let json_input = r#"{
        "lockfileVersion": 1,
        "importers": {
            ".": {
                "dependencies": {
                    "react": "18.2.0"
                }
            }
        },
        "packages": {
            "react@18.2.0": {
                "integrity": "sha512-4Z+FwM8Tq7bQ=="
            }
        }
    }"#;
    let _ = Lockfile::from_json(json_input);
}
