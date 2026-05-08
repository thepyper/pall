//! Runner test helper — shared utilities for all runner test files.
//!
//! Each machine group's test file (e.g., tests/counter_test.rs) imports
//! these helpers via `use super::helper::*;`.

use crate::stubs::*;

/// Result of running a machine until it reaches a goal or max ticks.
pub struct RunResult {
    /// Number of ticks taken to reach the goal.
    pub ticks_taken: u32,
    /// Final state string.
    pub final_state: String,
}

/// Run the machine's tick loop until the goal predicate is true or max_ticks exceeded.
///
/// # Arguments
/// * `max_ticks` — maximum number of ticks before giving up
/// * `delta_ms` — simulation time per tick
/// * `is_goal` — predicate that checks if the current state is the goal
///
/// # Returns
/// `Ok(RunResult)` if the goal was reached, `Err(msg)` if max ticks exceeded.
pub fn run_until(
    max_ticks: u32,
    delta_ms: u64,
    is_goal: fn(&Persistent) -> bool,
) -> Result<RunResult, String> {
    let mut state: Persistent = init();
    let mut ticks: u32 = 0;

    loop {
        if is_goal(&state) {
            return Ok(RunResult {
                ticks_taken: ticks,
                final_state: state.state.as_str().to_string(),
            });
        }

        if ticks >= max_ticks {
            return Err(format!(
                "Goal not reached after {} ticks. Final state: {}",
                max_ticks,
                state.state.as_str()
            ));
        }

        let tick_info = TickInfo { delta_ms };
        state = tick(&state, &tick_info).map_err(|e| e.message)?;
        ticks += 1;
    }
}

/// Run the machine for a fixed number of ticks, returning the final state.
///
/// # Arguments
/// * `num_ticks` — number of ticks to run
/// * `delta_ms` — simulation time per tick
///
/// # Returns
/// The `Persistent` state after the specified number of ticks.
pub fn run_for(num_ticks: u32, delta_ms: u64) -> Persistent {
    let mut state: Persistent = init();
    let mut ticks: u32 = 0;

    while ticks < num_ticks {
        let tick_info = TickInfo { delta_ms };
        state = tick(&state, &tick_info).expect("tick should succeed");
        ticks += 1;
    }

    state
}
