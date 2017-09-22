# Finite State Machine generator in Rust's macro

State machine consists of:

Name, initial state, List of States, List of Commands, list of State Nodes.

Each State Node contain: Name, State Context (optional), list of Command Reactions.

Each Command Reaction contain: Command to react on, user-defined code of reaction (optional) and next State of machine (optional).

# Some examples

Simplest state machine example:
```rust
#[macro_use] extern crate macro_machine;
declare_machine!(
    Simple(A) // Name and initial State
    states[A,B] // list of States
    commands[Next] // list of Commands
    (A: // State Node
        Next => B; // Command Reaction. Just change state to B
    )
    (B:
        Next => A; // And back to A
    )
);
```

So, now you can use state machine:

```rust
fn main() {
    use Simple::*;
    let mut machine = Simple::new();
    machine.execute(&Simple::Commands::Next).unwrap();
    machine.execute(&Simple::Commands::Next).unwrap();
}
```
You can add some intelligence to machine.

Each state can hold some data. On State change you can transmit some data between states.
It looks like you just create struct with some fields initialization:
```rust
#[macro_use] extern crate macro_machine;
declare_machine!(
    Simple(A{counter:0}) // Name and initial State with initial value
    states[A,B] // list of States
    commands[Next] // list of Commands
    (A context{counter:i16}: // State Node and this state context description with binding name
        Next {context.counter=context.counter+1}=> B{counter:context.counter}; // Command Reaction. Now on command Next we add 1 to our context. Also we change state to B and init it with our x value.
    )
    (B context{counter:i16}:
        Next {context.counter=context.counter+1}=> A{counter:context.counter};
    )
);
```

Let's check our state transmission:
```rust
fn main() {
    use Simple::*;
    let mut machine = Simple::new();
    
    // We are in state A and have our initial value 0
    assert!(match machine.state(){
        States::A{context}=> if context.counter == 0 {true} else {false},
        _=>false
    });
    machine.execute(&Simple::Commands::Next).unwrap();
    
    // We are in state B and have counter == 1
    assert!(match machine.state(){
        States::B{context}=> if context.counter == 1 {true} else {false},
        _=>false
    });
    machine.execute(&Simple::Commands::Next).unwrap();
    
    // Again in state A and have counter == 2
    assert!(match machine.state(){
        States::A{context}=> if context.counter == 2 {true} else {false},
        _=>false
    });
}
```

Also there is callbacks on each entrance and each leave of state.
 
```rust
#[macro_use] extern crate macro_machine;
declare_machine!(
    Simple(A{counter:0}) // Name and initial State with initial value
    states[A,B] // list of States
    commands[Next] // list of Commands
    (A context{counter:i16}: // State Node and this state context description with binding name
        >> {context.counter = context.counter+1;} // Execute when enter state A
        << {context.counter = context.counter+1;} // Execute when leave state A
        Next {context.counter=context.counter+1;} => B{counter:context.counter}; // Command Reaction. Now on command Next we add 1 to our context. Also we change state to B and init it with our x value.
    )
    (B context{counter:i16}:
        Next {context.counter=context.counter+1} => A{counter:context.counter};
    )
);
fn main() {
    use Simple::*;
    let mut machine = Simple::new();
    assert!(match machine.state(){
        // We are in state A and have value 1. Because Enter State callback executed.
        States::A{context}=> if context.counter == 1 {true} else {false},
        _=>false
    });
    machine.execute(&Simple::Commands::Next).unwrap();
    assert!(match machine.state(){
        // We are in state B and have counter == 3. Increment happen on User Code execution and execution of Leave state callback.
        States::B{context}=> {println!("context counter: {}", context.counter);if context.counter == 3 {true} else {false}},
        _=>false
    });
    machine.execute(&Simple::Commands::Next).unwrap();
    assert!(match machine.state(){
        // Again in state A and have counter == 5. Increment happen on User Code execution and on state A enter.
        States::A{context}=> if context.counter == 5 {true} else {false},
        _=>false
    });
}
```

# Changes
## 0.2.0
Changed behavior of Leave action. Now it execute before new State context creation.