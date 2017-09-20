#![deny(unused_must_use)]
//! State machine generator
//!
//! State machine consists of:
//! Name, initial state, List of States, List of Commands, list of States Nodes.
//! Each State Node contain: Name, State Context (optional), list of Command Reactions.
//! Each Command Reaction contain: Command to react on, user-defined code of reaction (optional) and
//!     next State of machine (optional).
//!
//! Simplest state machine example:
//!
//! ```
//! #[macro_use] extern crate macro_machine;
//!
//! declare_machine!(
//!     Simple(A) // Name and initial State
//!     states[A,B] // list of States
//!     commands[Next] // list of Commands
//!     (A: // State Node
//!         Next => B; // Command Reaction. Just change state to B
//!     )
//!     (B:
//!         Next => A;
//!     )
//! );
//!
//! # fn main() {
//! use Simple::*;
//!
//! let mut machine = Simple::new();
//! assert!(match machine.state(){States::A{..}=>true,_=>false});
//! machine.execute(&Simple::Commands::Next).unwrap();
//! assert!(match machine.state(){States::B{..}=>true,_=>false});
//! machine.execute(&Simple::Commands::Next).unwrap();
//! assert!(match machine.state(){States::A{..}=>true,_=>false});
//! # }
//! ```
//!
//! You can add some intelligence to machine:
//!
//! ```
//! #[macro_use] extern crate macro_machine;
//!
//! declare_machine!(
//!     Simple(A{counter:0}) // Name and initial State with initial value
//!     states[A,B] // list of States
//!     commands[Next] // list of Commands
//!     (A context{counter:i16}: // State Node and this state context description with binding name
//!         Next {context.counter=context.counter+1}=> B{counter:context.counter}; // Command Reaction. Now on command Next we add 1 to our context. Also we change state to B and init it with our x value.
//!     )
//!     (B context{counter:i16}:
//!         Next {context.counter=context.counter+1}=> A{counter:context.counter};
//!     )
//! );
//!
//! # fn main() {
//! use Simple::*;
//!
//! let mut machine = Simple::new();
//! assert!(match machine.state(){
//!     States::A{context}=> if context.counter == 0 {true} else {false}, // We are in state A and have our initial value 0
//!     _=>false
//! });
//! machine.execute(&Simple::Commands::Next).unwrap();
//! assert!(match machine.state(){
//!     States::B{context}=> if context.counter == 1 {true} else {false}, // We are in state B and have counter == 1
//!     _=>false
//! });
//! machine.execute(&Simple::Commands::Next).unwrap();
//! assert!(match machine.state(){
//!     States::A{context}=> if context.counter == 2 {true} else {false}, // Again in state A and have counter == 2
//!     _=>false
//! });
//! # }
//! ```
//! ```
//! #[macro_use] extern crate macro_machine;
//!
//! declare_machine!(
//!     Simple(A{counter:0}) // Name and initial State with initial value
//!     states[A,B] // list of States
//!     commands[Next] // list of Commands
//!     (A context{counter:i16}: // State Node and this state context description with binding name
//!         >> {context.counter = context.counter+1;} // Execute when enter state A
//!         << {context.counter = context.counter+1;} // Execute when leave state A
//!         Next {context.counter=context.counter+1;} => B{counter:context.counter}; // Command Reaction. Now on command Next we add 1 to our context. Also we change state to B and init it with our x value.
//!     )
//!     (B context{counter:i16}:
//!         Next {context.counter=context.counter+1} => A{counter:context.counter};
//!     )
//! );
//!
//! # fn main() {
//! use Simple::*;
//!
//! let mut machine = Simple::new();
//! assert!(match machine.state(){
//!
//!     // We are in state A and have value 1. Because Enter State callback executed.
//!
//!     States::A{context}=> if context.counter == 1 {true} else {false},
//!     _=>false
//! });
//! machine.execute(&Simple::Commands::Next).unwrap();
//! assert!(match machine.state(){
//!
//!     // We are in state B and have counter == 2. Increment happen on User Code execution. Execution of Leave state callback happen after we transfer data to the next state.
//!
//!     States::B{context}=> {println!("context counter: {}", context.counter);if context.counter == 2 {true} else {false}},
//!     _=>false
//! });
//! machine.execute(&Simple::Commands::Next).unwrap();
//! assert!(match machine.state(){
//!
//!     // Again in state A and have counter == 4. Increment happen on User Code execution and on state A enter.
//!
//!     States::A{context}=> if context.counter == 4 {true} else {false},
//!     _=>false
//! });
//! # }
//! ```
//!

#[macro_export]
macro_rules! declare_machine {

    // Initialize state by values
    (@inner next $new_state:ident{$($new_el:ident:$new_el_val:expr),*}) => (
        $new_state{$($new_el:$new_el_val),*}
    );

    // if state have no fields to initialize. Just for remove redundant curl braces.
    (@inner next $new_state:ident) => (
        $new_state{}
    );

    // If Event have user-defined code and move machine to new state. Execute code and return new state.
    (@inner command $cur:ident;$callback:block;$new_state:ident$({$($new_el:ident:$new_el_val:expr),*})*) => (
        {
            $callback;
            Some(States::$new_state{context: declare_machine!(@inner next $new_state$({$($new_el:$new_el_val),*})*)})
        }
    );

    // If Event have user-defined code and don't move machine to new state. Execute code and return __SameState__ .
    (@inner command $cur:ident;$callback:block;) => (
        {
            $callback;
            Some(States::__SameState__)
        }
    );

    // If Event have no user-defined code and move machine to new state. Just return new state.
    (@inner command $cur:ident; ;$new_state:ident$({$($new_el:ident:$new_el_val:expr),*})*) => (
        {
            Some(States::$new_state{context: declare_machine!(@inner next $new_state$({$($new_el:$new_el_val),*})*)})
        }
    );

    // If Event have nothing to do on event. Just return __SameState__.
    (@inner command $cur:ident ; ;) => (
        Some(States::__SameState__)
    );

    (@inner context $ss:ident $sel:ident)=>(let $sel = $ss;);
    (@inner context $ss:ident )=>();

    // Enter/Leave processors with and without user-defined code.
    (@inner >> $($sel:ident)* $income:block) => (
            fn enter(&mut self) -> Result<(), ()> {
                declare_machine!(@inner context self $($sel)*);
                $income
                Ok(())
            }
    );
    (@inner << $($sel:ident)* $outcome:block) => (
            fn leave(&mut self) -> Result<(), ()> {
                declare_machine!(@inner context self $($sel)*);
                $outcome
                Ok(())
            }
    );
    (@inner >> $($sel:ident)*) => (
            fn enter(&mut self) -> Result<(), ()> {
                Ok(())
            }
    );
    (@inner << $($sel:ident)*) => (
            fn leave(&mut self) -> Result<(), ()> {
                Ok(())
            }
    );

    // This structs keep user-defined contexts for states.
    (@inner params $state:ident {$($el:ident:$typ:ty);*}) => (
        #[derive(Debug)]
        #[derive(PartialEq)]
        #[derive(Copy)]
        #[derive(Clone)]
        pub struct $state {pub $($el:$typ),*}
    );
    (@inner params $state:ident) => (
        #[derive(Debug)]
        #[derive(PartialEq)]
        #[derive(Copy)]
        #[derive(Clone)]
        pub struct $state {}
    );
    (@inner initial $initial:ident{$($init_field:ident:$init_val:expr),*}) => ($initial{$($init_field: $init_val),*});
    (@inner initial $initial:ident) => ($initial{});

// Main pattern

(
    $machine:ident ($initial:ident$({$($init_field:ident:$init_val:expr),*})*)
    states[$($states:ident),*]
    commands[$($commands:ident),*]

    $(($state:ident $($sel:ident)*$({$($el:ident:$typ:ty);*})*:
        $(>> $income:block)*
        $(<< $outcome:block)*
        $($cmd:ident $($callback:block)* => $($new_state:ident$({$($new_el:ident:$new_el_val:expr),*})*)*;)*
    ))*
) => (
    #[allow(non_snake_case)]
    #[allow(unused_imports)]
    #[allow(dead_code)]
    #[allow(unused_variables)]
    mod $machine {
        use super::*;
        trait CanDoJob {
            fn do_job(&mut self, cmd: &Commands) -> Option<States>;
            fn leave(&mut self) -> Result<(), ()>;
            fn enter(&mut self) -> Result<(), ()>;
        }

        $(
        declare_machine!(@inner params $state $({$($el:$typ);*})*);

        impl CanDoJob for $state {
            fn do_job(&mut self, cmd: & Commands) -> Option<States> {
                declare_machine!(@inner context self $($sel)*);
                match *cmd {
                    $(Commands::$cmd => {declare_machine!(@inner command self;$($callback)*;$($new_state$({$($new_el:$new_el_val),*})*)*)})*
                    _ => None
                }
            }

            declare_machine!(@inner >> $($sel)* $($income)*);
            declare_machine!(@inner << $($sel)* $($outcome)*);
        }
        )*


        #[derive(Debug)]
        #[derive(PartialEq)]
        #[derive(Copy)]
        #[derive(Clone)]
        pub enum States {
            __SameState__,
            $($states {context: $states}),*
        }

        #[derive(Debug)]
        #[derive(PartialEq)]
        pub enum Commands {
            $($commands),*
        }

        pub struct Machine {
            state: States
        }
        pub fn new() -> Machine {
            let mut context = declare_machine!(@inner initial $initial $({$($init_field: $init_val),*})*);
            context.enter().unwrap();
            Machine{state: States::$initial{context: context}}
        }

        impl Machine {
            pub fn execute(&mut self, cmd: & Commands) -> Result<(),()>{
                match {
                    match self.state {
                        States::__SameState__ => None,
                        $(States::$state{ ref mut context } => context.do_job(cmd)),*
                    }
                } {
                    Some(x) => {
                        match x {
                            States::__SameState__ => {},
                            _ => {
                                self.change_state(x)
                            }
                        };Ok(())
                    },
                    None => {println!("Wrong operation {:?} for {:?} state!", cmd, self.state); Err(())}
                }
            }
            fn change_state(&mut self, new_state: States) {
                match self.state {
                    States::__SameState__ => Ok(()),
                    $(States::$state{ ref mut context } => context.leave()),*
                }.unwrap();
                self.state = new_state;
                match self.state {
                    States::__SameState__ => Ok(()),
                    $(States::$state{ ref mut context } => context.enter()),*
                }.unwrap();
            }
            pub fn state(&self) -> States {
                self.state.clone()
            }
        }
    }
)
}

#[cfg(test)]
mod tests {
    fn tes(x:i16) {
        println!("x:{}",x);
    }

    declare_machine!(
    Mach1 (New{x:0})

    states[New,InConfig,Operational]
    commands[Configure, ConfigureDone, Drop]

    ( New context{x: i16}:
        >> {println!("Enter {:?}", context)}
        << {println!("Leave {:?}", context)}
        Configure     {println!("In New with context val: {}", context.x);} =>     InConfig{x:context.x+1, y:0};
        ConfigureDone => New{x:0};
    )
    ( InConfig context{x:i16; y:i16}:
        >> {println!("Enter {:?}", context)}
        << {println!("Leave {:?}", context)}
        ConfigureDone {tes(context.x)}=> Operational;
    )
    ( Operational context:
        >> {println!("Enter {:?}", context)}
        << {println!("Leave {:?}", context)}
        ConfigureDone =>;
        Drop => New{x:0};
    )
    );

    declare_machine!(
    Mach2 (State1)

    states[State1,State2,State3]
    commands[ToState1, ToState2, ToState3]

    ( State1 :
        ToState2 => State2;
    )
    ( State2 :
        ToState3 => State3;
    )
    ( State3 :
        ToState1 => State1;
    )
    );

    #[test]
    fn test1() {
        let mut m = Mach1::new();
        m.execute(&Mach1::Commands::Configure).unwrap();
        m.execute(&Mach1::Commands::ConfigureDone).unwrap();
        m.execute(&Mach1::Commands::Drop).unwrap();
        m.execute(&Mach1::Commands::Configure).unwrap();
        m.execute(&Mach1::Commands::ConfigureDone).unwrap();
        m.execute(&Mach1::Commands::ConfigureDone).unwrap();
        m.execute(&Mach1::Commands::ConfigureDone).unwrap();
    }

    #[test]
    #[should_panic]
    fn test2() {
        let mut m = Mach2::new();
        m.execute(&Mach2::Commands::ToState2).unwrap();
        m.execute(&Mach2::Commands::ToState3).unwrap();
        m.execute(&Mach2::Commands::ToState1).unwrap();
        m.execute(&Mach2::Commands::ToState3).unwrap();
    }
}
