// The unit docs/decisions/layout.md places above both halves. It holds nothing
// beyond what a binary crate needs in order to compile, which is the same
// posture the twelve library crates that hold nothing are in.
//
// What this command takes, what it writes, and the three exit codes that
// distinguish success from a refused input and from an internal failure are
// issue #62's, and deciding any of them here would decide them in the place
// with the least argument attached.
fn main() {}
