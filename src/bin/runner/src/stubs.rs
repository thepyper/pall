// ── Stub modules for generated code imports ─────────────────────────────────

/// Minimal TickError to satisfy generated code imports.
pub mod error {
    use std::fmt;

    #[derive(Debug)]
    pub struct TickError {
        pub message: String,
    }

    impl fmt::Display for TickError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
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
// Generated files are created by: cargo run --bin gen-fixture
// Each machine gets its own types.rs and tick.rs under generated/{machine_id}/
// group.rs and mod.rs are generated once for all machines combined.

// ── counter_test machine stubs ───────────────────────────────────────────────

mod counter_test_types {
    include!("../generated/counter_test/types.rs");
}

mod counter_test_tick {
    include!("../generated/counter_test/tick.rs");
}

// ── traffic_light machine stubs ──────────────────────────────────────────────

mod traffic_light_types {
    include!("../generated/traffic_light/types.rs");
}

mod traffic_light_tick {
    include!("../generated/traffic_light/tick.rs");
}

// ── binary_counter machine stubs ─────────────────────────────────────────────

mod binary_counter_types {
    include!("../generated/binary_counter/types.rs");
}

mod binary_counter_tick {
    include!("../generated/binary_counter/tick.rs");
}

// ── Re-exports for convenience ──────────────────────────────────────────────
// Counter_test exports (used by counter_test.rs test)
pub use counter_test_types::Persistent;
pub use counter_test_types::State;
pub use counter_test_tick::{init, tick};

// Traffic_light exports (used by traffic_light.rs test)
pub use traffic_light_types::Persistent as TrafficLightPersistent;
pub use traffic_light_tick::init as traffic_light_init;
pub use traffic_light_tick::tick as traffic_light_tick_fn;

// Binary_counter exports (used by binary_counter.rs test)
pub use binary_counter_types::Persistent as BinaryCounterPersistent;
pub use binary_counter_tick::init as binary_counter_init;
pub use binary_counter_tick::tick as binary_counter_tick;

// Error type (used by helper and generated code)
pub use error::TickError;
