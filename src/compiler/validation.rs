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

    // Step 2.3: Unreachable transition detection
    for machine in machines {
        for (state_name, state) in &machine.states {
            let mut seen_always_true = false;
            for (index, transition) in state.transitions.iter().enumerate() {
                if seen_always_true {
                    errors.push(CompileError::new(
                        CompileErrorKind::UnreachableTransition,
                        format!(
                            "unreachable transition in machine '{}', state '{}', transition {}",
                            machine.id, state_name, index
                        ),
                    ));
                }
                if transition.when.is_none() {
                    seen_always_true = true;
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
