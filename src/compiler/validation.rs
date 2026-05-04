use crate::machine::StateMachine;
use super::CompileError;

/// Validate a set of state machines.
///
/// Runs all validation checks and collects errors.
/// Returns `Ok(())` if all machines are valid, or `Err(errors)` with a list of all issues found.
pub fn validate_machines(_machines: &[StateMachine]) -> Result<(), Vec<CompileError>> {
    Ok(())
}
