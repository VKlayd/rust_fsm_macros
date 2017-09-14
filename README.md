# Finite State Machine generator in Rust's macro

# Usage

# Define FSM:

    FSM!(
      fsm (State1{x:0})
    
      commands:(ToState1, ToState2)

      ( State1 context{x:i16}
        >>{}
        <<{}
        {ToState2;     ;             State2{x:context.x+1}}
      )
      ( State2 context{x:i16}
        >>{}
        <<{}
        {ToState1;     ;             State1{x:0}},
        {ToState2;     {println!("Already in State2!");};}
      )
    );

# Use it:

    let mut machine = fsm::new();
    machine.execute(&fsm::Commands::ToState2);
    machine.execute(&fsm::Commands::ToState2);
    machine.execute(&fsm::Commands::ToState1);
