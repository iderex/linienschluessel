//! Deliberate findings, for the measurement issue #52 asks for. Not for merging.
//!
//! Nothing here is called from anywhere and nothing here belongs in this
//! repository. It exists so that a run of the scanner has something to find, so
//! that which check goes red on a finding is measured rather than supposed.

use std::fs;
use std::path::PathBuf;

/// A path taken from outside the program and opened without being confined to a
/// directory the program chose.
pub fn read_whatever_the_caller_named() -> std::io::Result<String> {
    let named = std::env::var("LINIENSCHLUESSEL_INPUT").unwrap_or_default();
    let path = PathBuf::from(named);
    fs::read_to_string(path)
}

/// A secret written to the log in clear.
pub fn announce(user: &str, password: &str) {
    println!("reading the store as {user} with password {password}");
}
