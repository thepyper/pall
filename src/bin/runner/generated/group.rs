// Group types and tick for multi-machine compilation
// Auto-generated. Do not edit.

use serde::{Serialize, Deserialize};

use super::counter_test_types::Persistent;
use super::traffic_light_types::Persistent;
use super::binary_counter_types::Persistent;
use super::counter_test_tick::tick;
use super::traffic_light_tick::tick;
use super::binary_counter_tick::tick;
use super::TickInfo;
use super::error::TickError;

// ── GroupPersistent ────────────────────────────────────────────────────────
/// Holds persistent state for all compiled machines.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GroupPersistent {
    pub counter_test: Persistent,
    pub traffic_light: Persistent,
    pub binary_counter: Persistent,
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

    ys.traffic_light = tick(&ys.traffic_light, tick_info)?;

    ys.binary_counter = tick(&ys.binary_counter, tick_info)?;

    Ok(ys)
}
