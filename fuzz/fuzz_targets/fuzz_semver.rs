//! Fuzz target for semantic version and range parser.

use corex_semver::{Range, Version};

fn main() {
    let version_str = "1.2.3-alpha.1+build123";
    let _ = Version::parse(version_str);

    let range_str = "^1.2.0 || >=2.0.0 <3.0.0";
    let _ = Range::parse(range_str);
}
