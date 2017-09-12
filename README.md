# Finite State Machine generator in Rust's macro

# Usage

# Define FSM:

    FSM!(
      name: fsm (init: State1);
    
      states:(State1, State2);
      commands:(ToState1, ToState2);

      ( State1:
        {ToState2;     {};             State2}
      ),
      ( State2:
        {ToState1;     {};             State1},
        {ToState2;     {println!("Already in State2!");};             State2}
      )
    );

# Use it:

    let mut machine = fsm::new();
    machine.execute(&fsm::Commands::ToState2);
    machine.execute(&fsm::Commands::ToState2);
    machine.execute(&fsm::Commands::ToState1);
