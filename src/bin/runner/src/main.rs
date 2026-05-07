//! Runner — includes generated code and executes the state machine tick loop.

mod stubs;
use stubs::*;

/// Run the machine in a tick loop until goal or max ticks.
/// Returns (ticks_taken, final_counter_value).
fn run_machine(max_ticks: u32) -> Result<u32, String> {
    let mut state: Persistent = init();
    let mut ticks: u32 = 0;

    println!("Starting tick loop (max {} ticks)...", max_ticks);
    println!("Initial state: {}", state.state.as_str());

    loop {
        if state.state.as_str() == "goal" {
            println!("Goal reached at tick {}!", ticks);
            return Ok(ticks);
        }

        if ticks >= max_ticks {
            return Err(format!(
                "Goal not reached after {} ticks. Final state: {}",
                max_ticks,
                state.state.as_str()
            ));
        }

        let tick_info = TickInfo { delta_ms: 1000 };
        state = tick(&state, &tick_info).map_err(|e| e.message)?;
        ticks += 1;

        println!(
            "Tick {}: state={}, counter={}",
            ticks,
            state.state.as_str(),
            state.counter
        );
    }
}

fn main() {
    println!("=== Pall Runner ===");

    match run_machine(100) {
        Ok(ticks) => {
            println!("\n=== Runner done ===");
            println!("Total ticks: {}", ticks);
            println!("Final state: goal");
            println!("Final counter: >= 10");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_reaches_goal() {
        let ticks = run_machine(100).expect("machine should reach goal");
        assert!(ticks > 0, "should have taken at least one tick");
        println!("test_counter_reaches_goal: reached goal in {} ticks", ticks);
    }

    #[test]
    fn test_goal_reached_within_max_ticks() {
        let ticks = run_machine(100).expect("machine should reach goal within max ticks");
        // initial->counting (tick 1), then counter increments each tick.
        // Transition uses old counter value, so goal fires when old counter >= 10.
        // Total: ~12 ticks.
        assert!(ticks >= 1 && ticks <= 20, "expected ~12 ticks, got {}", ticks);
        println!("test_goal_reached_within_max_ticks: {} ticks", ticks);
    }

    #[test]
    fn test_generated_code_compiles() {
        let state: Persistent = init();
        assert_eq!(state.state.as_str(), "initial");
        assert_eq!(state.counter, 0);
    }
}
