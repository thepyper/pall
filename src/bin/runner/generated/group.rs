// Group types and tick for multi-machine compilation
// Auto-generated. Do not edit.

use serde::{Serialize, Deserialize};

use super::counter_test_types::Persistent;
use super::traffic_light_types::Persistent;
use super::binary_counter_types::Persistent;
use super::conditional_action_types::Persistent;
use super::arithmetic_ops_types::Persistent;
use super::assignment_ops_types::Persistent;
use super::counter_test_tick::tick;
use super::traffic_light_tick::tick;
use super::binary_counter_tick::tick;
use super::conditional_action_tick::tick;
use super::arithmetic_ops_tick::tick;
use super::assignment_ops_tick::tick;
use super::TickInfo;
use super::error::TickError;

// ── GroupPersistent ────────────────────────────────────────────────────────
/// Holds persistent state for all compiled machines.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GroupPersistent {
    pub counter_test: Persistent,
    pub traffic_light: Persistent,
    pub binary_counter: Persistent,
    pub conditional_action: Persistent,
    pub arithmetic_ops: Persistent,
    pub assignment_ops: Persistent,
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

    ys.conditional_action = tick(&ys.conditional_action, tick_info)?;

    ys.arithmetic_ops = tick(&ys.arithmetic_ops, tick_info)?;

    ys.assignment_ops = tick(&ys.assignment_ops, tick_info)?;

    Ok(ys)
}
