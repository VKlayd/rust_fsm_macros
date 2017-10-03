# Finite State Machine generator in Rust's macro

## Overview

With this macro you can easily implement Finite State Machine in declarative way.

State machine consists of:

* Name
* Initial state
* List of **states**
* List of **commands**
* List of **state nodes**

Each **state node** contains:

* State
* Context (optional)
* List of **command reactions**

Each **command reaction** contains:

* Command to react on
* User-defined code of reaction (optional)
* Next state of machine (optional)

## Working example to begin with

Let's say we'd like to implement such machine:

![FSM example](http://www.plantuml.com/plantuml/proxy?src=https://gist.githubusercontent.com/goldenreign/e363fe08501362d9618f2012f1ddfe2f/raw/07bb4bc604a204e7bcef38154135ee8a14c10f5b/gistfile1.puml "FSM example")

Corresponding code will look like:

```rust
#[macro_use] extern crate macro_machine;
declare_machine!(
    MyMachine(A {counter: 0}) // Name and initial state with initial value
    states[A,B] // List of states
    commands[Next] // List of commands
    (A context{counter: i16}: // State node and this state context description with name binding
        >> { // Executed on state A enter
            println!("Enter A: {:?}", context);
            context.counter = context.counter + 1;
        }
        << { // Executed on state A leave
            println!("Leave A: {:?}", context);
            context.counter = context.counter + 1;
        }
        Next {
            println!("Next in A: {:?}", context);
            context.counter = context.counter + 1;
        } => B {counter: context.counter}; // Command Reaction. Now on command Next we add 1 to our context. Also we change state to B and init it with our counter value.
    )
    (B context{counter: i16}:
        >> {
            println!("Enter B: {:?}", context);
            context.counter = context.counter + 1;
        }
        << {
            println!("Leave B: {:?}", context);
            context.counter = context.counter + 1;
        }
        Next {
            println!("Next in B: {:?}", context);
            context.counter = context.counter + 1;
        } => A {counter: context.counter};
    )
);

fn main() {
    use MyMachine::*;
    let mut machine = MyMachine::new();
    machine.execute(&MyMachine::Commands::Next).unwrap();
    machine.execute(&MyMachine::Commands::Next).unwrap();
}
```

## Longer explanation

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

Example of Machine-scoped context. This context exist in machine life-time.

Let's count machine's state changes:

```rust
#[macro_use] extern crate macro_machine;
declare_machine!(
    Simple machine_context{counter: i16} (A) // Declare machine scoped context
    states[A,B]
    commands[Next]
    (A :
        >> {machine_context.counter=machine_context.counter+1;} // Add 1 when enter in state
        Next => B; // Just switch to other state
    )
    (B :
        >> {machine_context.counter=machine_context.counter+1;}
        Next => A;
    )
);
fn main() {
    use Simple::*;
    let mut machine = Simple::new(0); // Create machine and initiate machine context by 0
    let context = machine.get_inner_context();
    assert!(context.counter == 1);
    machine.execute(&Simple::Commands::Next).unwrap();
    let context = machine.get_inner_context();
    assert!(context.counter == 2);
    machine.execute(&Simple::Commands::Next).unwrap();
    let context = machine.get_inner_context();
    assert!(context.counter == 3);
}
```

## Changelog

### 0.2.0

* Changed behavior of Leave action. Now it execute before new State context creation.
* Add machine-scoped context. It can be used by all callbacks inside machine. Data in this context have machine's life-time.
