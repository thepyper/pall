


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
