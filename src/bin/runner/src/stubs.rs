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

// ── conditional_action machine stubs ─────────────────────────────────────────

mod conditional_action_types {
    include!("../generated/conditional_action/types.rs");
}

mod conditional_action_tick {
    include!("../generated/conditional_action/tick.rs");
}

// ── arithmetic_ops machine stubs ─────────────────────────────────────────────

mod arithmetic_ops_types {
    include!("../generated/arithmetic_ops/types.rs");
}

mod arithmetic_ops_tick {
    include!("../generated/arithmetic_ops/tick.rs");
}

// ── assignment_ops machine stubs ─────────────────────────────────────────────

mod assignment_ops_types {
    include!("../generated/assignment_ops/types.rs");
}

mod assignment_ops_tick {
    include!("../generated/assignment_ops/tick.rs");
}

// ── logic_ops machine stubs ──────────────────────────────────────────────────

mod logic_ops_types {
    include!("../generated/logic_ops/types.rs");
}

mod logic_ops_tick {
    include!("../generated/logic_ops/tick.rs");
}

// ── bitwise_ops machine stubs ────────────────────────────────────────────────

mod bitwise_ops_types {
    include!("../generated/bitwise_ops/types.rs");
}

mod bitwise_ops_tick {
    include!("../generated/bitwise_ops/tick.rs");
}

// ── expression_precedence machine stubs ──────────────────────────────────────

mod expression_precedence_types {
    include!("../generated/expression_precedence/types.rs");
}

mod expression_precedence_tick {
    include!("../generated/expression_precedence/tick.rs");
}

// type_casting machine stubs
mod type_casting_types {
    include!("../generated/type_casting/types.rs");
}

mod type_casting_tick {
    include!("../generated/type_casting/tick.rs");
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

// Conditional_action exports (used by conditional_action.rs test)
pub use conditional_action_types::Persistent as ConditionalActionPersistent;
pub use conditional_action_tick::init as conditional_action_init;
pub use conditional_action_tick::tick as conditional_action_tick;

// Arithmetic_ops exports (used by arithmetic_ops.rs test)
pub use arithmetic_ops_types::Persistent as ArithmeticOpsPersistent;
pub use arithmetic_ops_tick::init as arithmetic_ops_init;
pub use arithmetic_ops_tick::tick as arithmetic_ops_tick;

// Assignment_ops exports (used by assignment_ops.rs test)
pub use assignment_ops_types::Persistent as AssignmentOpsPersistent;
pub use assignment_ops_tick::init as assignment_ops_init;
pub use assignment_ops_tick::tick as assignment_ops_tick;

// Logic_ops exports (used by logic_ops.rs test)
pub use logic_ops_types::Persistent as LogicOpsPersistent;
pub use logic_ops_tick::init as logic_ops_init;
pub use logic_ops_tick::tick as logic_ops_tick;

// Bitwise_ops exports (used by bitwise_ops.rs test)
pub use bitwise_ops_types::Persistent as BitwiseOpsPersistent;
pub use bitwise_ops_tick::init as bitwise_ops_init;
pub use bitwise_ops_tick::tick as bitwise_ops_tick;

// Expression_precedence exports (used by expression_precedence.rs test)
pub use expression_precedence_types::Persistent as ExpressionPrecedencePersistent;
pub use expression_precedence_tick::init as expression_precedence_init;
pub use expression_precedence_tick::tick as expression_precedence_tick;

// Type_casting exports (used by type_casting.rs test)
pub use type_casting_types::Persistent as TypeCastingPersistent;
pub use type_casting_tick::init as type_casting_init;
pub use type_casting_tick::tick as type_casting_tick;

// Error type (used by helper and generated code)
pub use error::TickError;
