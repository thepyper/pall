// Group types and tick for multi-machine compilation
// Auto-generated. Do not edit.

use serde::{Serialize, Deserialize};

use super::type_casting_types::Persistent;
use super::type_casting_tick::tick;
use super::TickInfo;
use super::error::TickError;

// ── GroupPersistent ────────────────────────────────────────────────────────
/// Holds persistent state for all compiled machines.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GroupPersistent {
    pub type_casting: Persistent,
}

// ── Group Tick Function ────────────────────────────────────────────────────
/// Execute one tick across all machines.
/// Phase 1: propagate links. Phase 2: tick each machine.
pub fn group_tick(
    xs: &GroupPersistent,
    tick_info: &TickInfo,
) -> Result<GroupPersistent, TickError> {
    let mut ys = xs.clone();


    ys.type_casting = tick(&ys.type_casting, tick_info)?;

    Ok(ys)
}
