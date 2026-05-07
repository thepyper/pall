// Group types and tick for multi-machine compilation
// Auto-generated. Do not edit.

use serde::{Serialize, Deserialize};

use super::counter_test_types::Persistent;
use super::counter_test_tick::tick;
use super::TickInfo;
use super::error::TickError;

// ── GroupPersistent ────────────────────────────────────────────────────────
/// Holds persistent state for all compiled machines.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GroupPersistent {
    pub counter_test: Persistent,
}

// ── Group Tick Function ────────────────────────────────────────────────────
/// Execute one tick across all machines.
/// Phase 1: propagate links. Phase 2: tick each machine.
pub fn group_tick(
    xs: &GroupPersistent,
    tick_info: &TickInfo,
) -> Result<GroupPersistent, TickError> {
    let mut ys = xs.clone();


    ys.counter_test = tick(&ys.counter_test, tick_info)?;

    Ok(ys)
}
