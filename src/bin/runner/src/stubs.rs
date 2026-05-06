// ── Stub modules for generated code imports ─────────────────────────────────

/// Minimal TickError to satisfy generated code imports.
pub mod error {
    use std::fmt;

    #[derive(Debug)]
    pub struct TickError {
        pub message: String,
    }

    impl fmt::Display for TickError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl std::error::Error for TickError {}
}

/// Minimal TickInfo to satisfy generated code imports.
pub struct TickInfo {
    pub delta_ms: u64,
}

// ── Include generated code ──────────────────────────────────────────────────

mod counter_test_types {
    include!("../generated/counter_test/types.rs");
}

mod counter_test_tick {
    include!("../generated/counter_test/tick.rs");
}

mod group {
    include!("../generated/group.rs");
}

// ── Re-exports for convenience ──────────────────────────────────────────────

pub use counter_test_types::{Persistent, State, Update};
pub use counter_test_tick::{init, tick};

// ── Helper: apply Update to Persistent ──────────────────────────────────────

/// Merge an Update into a Persistent, overwriting Some values.
pub fn apply_update(state: &mut Persistent, update: &Update) {
    if let Some(v) = update.counter {
        state.counter = v;
    }
    if let Some(s) = update.state {
        state.state = s;
        state.state_name = s.as_str().to_string();
    }
}
