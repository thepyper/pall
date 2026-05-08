//! Runner — includes generated code and runs tests.
//!
//! Generated machine code is included via `include!` macros in `stubs.rs`.
//! Tests are in `tests/` subdirectory (one file per machine group).
//!
//! Module re-exports at crate root level for generated code imports.
//! Generated tick.rs expects: `super::super::TickInfo` and `super::super::error::TickError`
//! from the runner crate root.

mod stubs;

// Re-export TickInfo and error module at crate root for generated code.
// The generated tick.rs template uses: use super::super::TickInfo;
//                                and: use super::super::error::TickError;
pub use stubs::TickInfo;
pub use stubs::error;

#[cfg(test)]
mod tests;

fn main() {
    println!("Pall Runner — use 'cargo test' to run runner tests.");
}
