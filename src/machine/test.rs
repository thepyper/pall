


#[test]
fn test_deserialize_machine_minimal()
{
    use crate::machine::StateMachine;

    let _x: StateMachine = serde_yaml::from_str(r###"
id: test_machine
states:
    initial:
        actions: []
        transitions: []
"###).unwrap();
}

#[test]
fn test_deserialize_machine()
{
    use crate::machine::StateMachine;

    let _x: StateMachine = serde_yaml::from_str(r###"
id: test_machine
states:
    initial:
        actions: []
        transitions: []

"###).unwrap();
}

#[test]
fn test_roundtrip_expression_statement_link()
{
    use crate::machine::StateMachine;

    let yaml = r###"
id: roundtrip_test
states:
  initial:
    actions: []
    transitions:
      - when: "0xff + 3.14 == a"
        do:
          - "result += 0o17"
        target: "next"
  next:
    actions: []
    transitions: []
inputs:
  my_input:
    type: Bool
    link: "source.my_signal"
"###;

    let machine: StateMachine = serde_yaml::from_str(yaml).expect("Failed to deserialize");

    // Verify we got the right machine
    assert_eq!(machine.id, "roundtrip_test");

    // Check state transitions
    let initial = machine.states.get("initial").expect("initial state missing");
    assert_eq!(initial.transitions.len(), 1);

    let trans = &initial.transitions[0];
    assert_eq!(trans.target, "next");

    // Verify FullExpression.raw preserved
    let when = trans.when.as_ref().expect("when expression missing");
    assert_eq!(when.raw, "0xff + 3.14 == a");

    // Verify FullStatement.raw preserved
    assert_eq!(trans.r#do.len(), 1);
    let stmt = &trans.r#do[0];
    assert_eq!(stmt.raw, "result += 0o17");

    // Verify link parsing
    let input = machine.inputs.get("my_input").expect("my_input missing");
    let link = input.link.as_ref().expect("link missing");
    assert_eq!(link.id, "source");
    assert_eq!(link.output, "my_signal");

    // Round-trip: serialize back to YAML
    let serialized = serde_yaml::to_string(&machine).expect("Failed to serialize");

    // Re-parse and verify values match
    let machine2: StateMachine = serde_yaml::from_str(&serialized).expect("Failed to re-parse");
    let initial2 = machine2.states.get("initial").expect("initial state missing after round-trip");
    let trans2 = &initial2.transitions[0];
    assert_eq!(trans2.target, "next");
}
