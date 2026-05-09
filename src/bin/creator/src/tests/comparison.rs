//! Shared comparison helper for YAML vs programmatic StateMachine equality.
//!
//! Each new machine test file imports this via `use super::comparison::compare_state_machines;`

use std::collections::HashMap;

use pall::machine::{
    Action, Constant, FullExpression, FullStatement, Input, Signal, State,
    StateMachine, Timer, Transition, Variable,
};

/// Compare two StateMachines for semantic (content) equality.
/// HashMaps are compared by key-value content, ignoring order.
pub fn compare_state_machines(a: &StateMachine, b: &StateMachine) -> Result<(), String> {
    if a.id != b.id {
        return Err(format!("id mismatch: '{}' != '{}'", a.id, b.id));
    }
    if a.initial != b.initial {
        return Err(format!(
            "initial mismatch: '{:?}' != '{:?}'",
            a.initial, b.initial
        ));
    }

    compare_state_map(&a.states, &b.states, "states")?;
    compare_var_map(&a.variables, &b.variables, "variables")?;
    compare_input_map(&a.inputs, &b.inputs, "inputs")?;
    compare_signal_map(&a.signals, &b.signals, "signals")?;
    compare_timer_map(&a.timers, &b.timers, "timers")?;
    compare_const_map(&a.constants, &b.constants, "constants")?;

    Ok(())
}

fn compare_state_map(
    a: &HashMap<String, State>,
    b: &HashMap<String, State>,
    field: &str,
) -> Result<(), String> {
    if a.len() != b.len() {
        return Err(format!("{} size mismatch: {} != {}", field, a.len(), b.len()));
    }
    for (key, val_a) in a {
        match b.get(key) {
            Some(val_b) => compare_states(val_a, val_b, field)?,
            None => return Err(format!("{} missing key: '{}'", field, key)),
        }
    }
    Ok(())
}

fn compare_states(a: &State, b: &State, parent: &str) -> Result<(), String> {
    let pfx = format!("{}.states[{}]", parent, "???");

    if a.actions.len() != b.actions.len() {
        return Err(format!(
            "{} actions count mismatch: {} != {}",
            parent, a.actions.len(), b.actions.len()
        ));
    }
    for (i, (a_act, b_act)) in a.actions.iter().zip(b.actions.iter()).enumerate() {
        compare_actions(a_act, b_act, &format!("{}.actions[{}]", pfx, i))?;
    }

    if a.transitions.len() != b.transitions.len() {
        return Err(format!(
            "{} transitions count mismatch: {} != {}",
            parent, a.transitions.len(), b.transitions.len()
        ));
    }
    for (i, (a_trans, b_trans)) in a.transitions.iter().zip(b.transitions.iter()).enumerate() {
        compare_transitions(a_trans, b_trans, &format!("{}.transitions[{}]", pfx, i))?;
    }

    Ok(())
}

fn compare_actions(a: &Action, b: &Action, parent: &str) -> Result<(), String> {
    match (&a.when, &b.when) {
        (Some(ae), Some(be)) => compare_expressions(ae, be, parent)?,
        (None, None) => {}
        (Some(_), None) => return Err(format!("{} when: Some != None", parent)),
        (None, Some(_)) => return Err(format!("{} when: None != Some", parent)),
    }

    if a.r#do.len() != b.r#do.len() {
        return Err(format!(
            "{} do count mismatch: {} != {}",
            parent, a.r#do.len(), b.r#do.len()
        ));
    }
    for (i, (a_stmt, b_stmt)) in a.r#do.iter().zip(b.r#do.iter()).enumerate() {
        compare_statements(a_stmt, b_stmt, &format!("{}.do[{}]", parent, i))?;
    }

    Ok(())
}

fn compare_transitions(a: &Transition, b: &Transition, parent: &str) -> Result<(), String> {
    match (&a.when, &b.when) {
        (Some(ae), Some(be)) => compare_expressions(ae, be, parent)?,
        (None, None) => {}
        (Some(_), None) => return Err(format!("{} when: Some != None", parent)),
        (None, Some(_)) => return Err(format!("{} when: None != Some", parent)),
    }

    if a.r#do.len() != b.r#do.len() {
        return Err(format!(
            "{} do count mismatch: {} != {}",
            parent, a.r#do.len(), b.r#do.len()
        ));
    }
    for (i, (a_stmt, b_stmt)) in a.r#do.iter().zip(b.r#do.iter()).enumerate() {
        compare_statements(a_stmt, b_stmt, &format!("{}.do[{}]", parent, i))?;
    }

    if a.target != b.target {
        return Err(format!(
            "{} target mismatch: '{}' != '{}'",
            parent, a.target, b.target
        ));
    }

    Ok(())
}

fn compare_expressions(a: &FullExpression, b: &FullExpression, parent: &str) -> Result<(), String> {
    if a.raw != b.raw {
        return Err(format!(
            "{} expression raw mismatch: '{}' != '{}'",
            parent, a.raw, b.raw
        ));
    }
    if a.expression != b.expression {
        return Err(format!("{} expression content mismatch", parent));
    }
    Ok(())
}

fn compare_statements(a: &FullStatement, b: &FullStatement, parent: &str) -> Result<(), String> {
    if a.raw != b.raw {
        return Err(format!(
            "{} statement raw mismatch: '{}' != '{}'",
            parent, a.raw, b.raw
        ));
    }
    if a.statement != b.statement {
        return Err(format!("{} statement content mismatch", parent));
    }
    Ok(())
}

fn compare_var_map(
    a: &HashMap<String, Variable>,
    b: &HashMap<String, Variable>,
    parent: &str,
) -> Result<(), String> {
    if a.len() != b.len() {
        return Err(format!("{} size mismatch: {} != {}", parent, a.len(), b.len()));
    }
    for (key, val_a) in a {
        match b.get(key) {
            Some(val_b) => {
                if val_a != val_b {
                    return Err(format!("{}[{}] mismatch", parent, key));
                }
            }
            None => return Err(format!("{} missing key: '{}'", parent, key)),
        }
    }
    Ok(())
}

fn compare_input_map(
    a: &HashMap<String, Input>,
    b: &HashMap<String, Input>,
    parent: &str,
) -> Result<(), String> {
    if a.len() != b.len() {
        return Err(format!("{} size mismatch: {} != {}", parent, a.len(), b.len()));
    }
    for (key, val_a) in a {
        match b.get(key) {
            Some(val_b) => {
                if val_a != val_b {
                    return Err(format!("{}[{}] mismatch", parent, key));
                }
            }
            None => return Err(format!("{} missing key: '{}'", parent, key)),
        }
    }
    Ok(())
}

fn compare_signal_map(
    a: &HashMap<String, Signal>,
    b: &HashMap<String, Signal>,
    parent: &str,
) -> Result<(), String> {
    if a.len() != b.len() {
        return Err(format!("{} size mismatch: {} != {}", parent, a.len(), b.len()));
    }
    for (key, val_a) in a {
        match b.get(key) {
            Some(val_b) => {
                if val_a != val_b {
                    return Err(format!("{}[{}] mismatch", parent, key));
                }
            }
            None => return Err(format!("{} missing key: '{}'", parent, key)),
        }
    }
    Ok(())
}

fn compare_timer_map(
    a: &HashMap<String, Timer>,
    b: &HashMap<String, Timer>,
    parent: &str,
) -> Result<(), String> {
    if a.len() != b.len() {
        return Err(format!("{} size mismatch: {} != {}", parent, a.len(), b.len()));
    }
    for (key, val_a) in a {
        match b.get(key) {
            Some(val_b) => {
                if val_a != val_b {
                    return Err(format!("{}[{}] mismatch", parent, key));
                }
            }
            None => return Err(format!("{} missing key: '{}'", parent, key)),
        }
    }
    Ok(())
}

fn compare_const_map(
    a: &HashMap<String, Constant>,
    b: &HashMap<String, Constant>,
    parent: &str,
) -> Result<(), String> {
    if a.len() != b.len() {
        return Err(format!("{} size mismatch: {} != {}", parent, a.len(), b.len()));
    }
    for (key, val_a) in a {
        match b.get(key) {
            Some(val_b) => {
                if val_a != val_b {
                    return Err(format!("{}[{}] mismatch", parent, key));
                }
            }
            None => return Err(format!("{} missing key: '{}'", parent, key)),
        }
    }
    Ok(())
}
