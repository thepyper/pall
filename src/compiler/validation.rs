use std::collections::HashSet;

use crate::machine::StateMachine;
use super::{CompileError, CompileErrorKind};

/// Validate a set of state machines.
///
/// Runs all validation checks and collects errors.
/// Returns `Ok(())` if all machines are valid, or `Err(errors)` with a list of all issues found.
pub fn validate_machines(machines: &[StateMachine]) -> Result<(), Vec<CompileError>> {
    let mut errors = Vec::new();

    // Step 2.2: Duplicate machine ID validation
    let mut seen_ids = HashSet::new();
    for machine in machines {
        if !seen_ids.insert(machine.id.clone()) {
            errors.push(CompileError::new(
                CompileErrorKind::DuplicateMachineId,
                format!("duplicate machine id: '{}'", machine.id),
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
