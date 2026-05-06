// Group types and tick for multi-machine compilation
// Auto-generated. Do not edit.

use serde::{Serialize, Deserialize};

use super::counter_test_types::{Persistent, Update};
use super::counter_test_tick::tick;
use super::TickInfo;
use super::error::TickError;

// ── GroupPersistent ────────────────────────────────────────────────────────
/// Holds persistent state for all compiled machines.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GroupPersistent {
    pub counter_test: Persistent,
}

// ── GroupUpdate ────────────────────────────────────────────────────────────
/// Holds updates from all machines after one tick.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GroupUpdate {
    pub counter_test: Update,
}

// ── Group Tick Function ────────────────────────────────────────────────────
/// Execute one tick across all machines.
/// Phase 1: propagate links. Phase 2: tick each machine.
pub fn group_tick(
    xs: &GroupPersistent,
    tick_info: &TickInfo,
) -> Result<GroupUpdate, TickError> {
    let mut ys = GroupUpdate {
        counter_test: Update::default(),
    };


    {
        let x = &xs.counter_test;
        let result = tick(x, tick_info)?;
        ys.counter_test = result;
    }

    Ok(ys)
}
