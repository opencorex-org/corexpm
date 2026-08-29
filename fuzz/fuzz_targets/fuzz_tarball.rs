//! Fuzz target for safe tarball extraction stream.

use corex_fetch::extract_tarball_stream;
use std::io::Cursor;

fn main() {
    let dummy_tarball = Vec::new();
    let temp_dir = std::env::temp_dir();
    let reader = Cursor::new(dummy_tarball);
    let _ = extract_tarball_stream(reader, &temp_dir, None);
}
