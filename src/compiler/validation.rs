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

    // Step 2.4: Missing state reference validation
    for machine in machines {
        for (state_name, state) in &machine.states {
            for (index, transition) in state.transitions.iter().enumerate() {
                if !machine.states.contains_key(&transition.target) {
                    errors.push(CompileError::new(
                        CompileErrorKind::MissingStateReference,
                        format!(
                            "missing state reference in machine '{}', state '{}', transition {}: target '{}' does not exist",
                            machine.id, state_name, index, transition.target
                        ),
                    ));
                }
            }
        }
    }

    // Step 2.5: Link reference validation
    // Build a map: (source_machine_id, var_name) -> target_machine_id
    // Then verify each link points to valid machines and variables
    let machine_by_id: std::collections::HashMap<&str, &StateMachine> =
        machines.iter().map(|m| (m.id.as_str(), m)).collect();

    for machine in machines {
        for (input_name, input) in &machine.inputs {
            if let Some(link) = &input.link {
                // Check source machine exists
                let source_machine = match machine_by_id.get(link.id.as_str()) {
                    Some(m) => m,
                    None => {
                        errors.push(CompileError::new(
                            CompileErrorKind::InvalidLink,
                            format!(
                                "invalid link in machine '{}', input '{}': source machine '{}' does not exist",
                                machine.id, input_name, link.id
                            ),
                        ));
                        continue;
                    }
                };

                // Check source variable exists and has output: true
                let source_has_var =
                    check_output_var_exists(source_machine, &link.output);
                if !source_has_var {
                    errors.push(CompileError::new(
                        CompileErrorKind::InvalidLink,
                        format!(
                            "invalid link in machine '{}', input '{}': source machine '{}', variable '{}' not found or not flagged as output",
                            machine.id, input_name, link.id, link.output
                        ),
                    ));
                }
            }
        }
    }

    // Step 2.6: Timer type validation
    for machine in machines {
        for (timer_name, timer) in &machine.timers {
            let is_numeric = matches!(
                timer.r#type,
                crate::machine::Type::U8
                    | crate::machine::Type::U16
                    | crate::machine::Type::U32
                    | crate::machine::Type::U64
                    | crate::machine::Type::I8
                    | crate::machine::Type::I16
                    | crate::machine::Type::I32
                    | crate::machine::Type::I64
                    | crate::machine::Type::F32
                    | crate::machine::Type::F64
            );
            let is_float = matches!(
                timer.r#type,
                crate::machine::Type::F32 | crate::machine::Type::F64
            );

            if !is_numeric {
                errors.push(CompileError::new(
                    CompileErrorKind::InvalidTimerType,
                    format!(
                        "invalid timer type in machine '{}', timer '{}': must be numeric, got {:?}",
                        machine.id, timer_name, timer.r#type
                    ),
                ));
            } else if is_float {
                // Warning: float is not an error but not ideal
                // We'll add a warning variant if needed later; for now just a note
            }
        }
    }

    // Step 2.7: Signal expression type validation (basic)
    for machine in machines {
        for (signal_name, signal) in &machine.signals {
            // Basic check: the expression must exist and be valid
            // Full type compatibility check will be done during code generation
            // For now, we verify the signal has a defined type
            match signal.r#type {
                crate::machine::Type::String
                | crate::machine::Type::Bool
                | crate::machine::Type::U8
                | crate::machine::Type::U16
                | crate::machine::Type::U32
                | crate::machine::Type::U64
                | crate::machine::Type::I8
                | crate::machine::Type::I16
                | crate::machine::Type::I32
                | crate::machine::Type::I64
                | crate::machine::Type::F32
                | crate::machine::Type::F64 => {
                    // Valid signal type — further type checking happens in codegen
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

/// Check if a variable exists on the machine with output: true.
fn check_output_var_exists(machine: &StateMachine, var_name: &str) -> bool {
    // Check inputs
    if let Some(input) = machine.inputs.get(var_name) {
        if input.output {
            return true;
        }
    }
    // Check variables
    if let Some(variable) = machine.variables.get(var_name) {
        if variable.output {
            return true;
        }
    }
    // Check signals
    if let Some(signal) = machine.signals.get(var_name) {
        if signal.output {
            return true;
        }
    }
    // Check constants
    if let Some(constant) = machine.constants.get(var_name) {
        if constant.output {
            return true;
        }
    }
    false
}
