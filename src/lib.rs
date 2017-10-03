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
//!     Simple (A) // Name and initial State
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
//!     Simple (A{counter:0}) // Name and initial State with initial value
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
//!     Simple (A{counter:0}) // Name and initial State with initial value
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
//!     // We are in state B and have counter == 3. Increment happen on User Code execution. Execution of Leave state callback happen after we transfer data to the next state.
//!
//!     States::B{context}=> {println!("context counter: {}", context.counter);if context.counter == 3 {true} else {false}},
//!     _=>false
//! });
//! machine.execute(&Simple::Commands::Next).unwrap();
//! assert!(match machine.state(){
//!
//!     // Again in state A and have counter == 5. Increment happen on User Code execution and on state A enter.
//!
//!     States::A{context}=> if context.counter == 5 {true} else {false},
//!     _=>false
//! });
//! # }
//! ```
//!
//! Example of Machine-scoped context. This context exist in machine life-time.
//!
//! Lets count machine's state changes:
//!
//! ```
//! #[macro_use] extern crate macro_machine;
//!
//! declare_machine!(
//!     Simple machine_context{counter: i16} (A) // Declare machine scoped context
//!     states[A,B]
//!     commands[Next]
//!     (A :
//!         >> {machine_context.counter=machine_context.counter+1;} // Add 1 when enter in state
//!         Next => B; // Just switch to other state
//!     )
//!     (B :
//!         >> {machine_context.counter=machine_context.counter+1;}
//!         Next => A;
//!     )
//! );
//!
//! # fn main() {
//! use Simple::*;
//!
//! let mut machine = Simple::new(0);
//! let context = machine.get_inner_context();
//! assert!(context.counter == 1);
//! machine.execute(&Simple::Commands::Next).unwrap();
//! let context = machine.get_inner_context();
//! assert!(context.counter == 2);
//! machine.execute(&Simple::Commands::Next).unwrap();
//! let context = machine.get_inner_context();
//! assert!(context.counter == 3);
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
    (@inner command @$glob_context:ident@ $sel:ident:$cur:ident;$callback:block;$new_state:ident$({$($new_el:ident:$new_el_val:expr),*})*) => (
        {
            declare_machine!(@inner context $sel $cur);
            $callback;
            $cur.leave($glob_context).unwrap();
            Some(States::$new_state{context: declare_machine!(@inner next $new_state$({$($new_el:$new_el_val),*})*)})
        }
    );

    // If Event have user-defined code and don't move machine to new state. Execute code and return __SameState__ .
    (@inner command @$glob_context:ident@ $sel:ident:$cur:ident;$callback:block;) => (
        {
            declare_machine!(@inner context $sel $cur);
            $callback;
            Some(States::__SameState__)
        }
    );

    // If Event have no user-defined code and move machine to new state. Just return new state.
    (@inner command @$glob_context:ident@ $sel:ident:$cur:ident; ;$new_state:ident$({$($new_el:ident:$new_el_val:expr),*})*) => (
        {
            declare_machine!(@inner context $sel $cur);
            $cur.leave($glob_context).unwrap();
            Some(States::$new_state{context: declare_machine!(@inner next $new_state$({$($new_el:$new_el_val),*})*)})
        }
    );

    // If Event have nothing to do on event. Just return __SameState__.
    (@inner command @$glob_context:ident@ $sel:ident:$cur:ident ; ;) => (
        Some(States::__SameState__)
    );

    // If Event have user-defined code and move machine to new state. Execute code and return new state.
    (@inner command @$glob_context:ident@ $sel:ident:$cur:ident;$callback:block;$new_state:ident$({$($new_el:ident:$new_el_val:expr),*})*) => (
        {
            declare_machine!(@inner context $sel $cur);
            $callback;
            $cur.leave($glob_context).unwrap();
            Some(States::$new_state{context: declare_machine!(@inner next $new_state$({$($new_el:$new_el_val),*})*)})
        }
    );

    // If Event have user-defined code and don't move machine to new state. Execute code and return __SameState__ .
    (@inner command @$glob_context:ident@ $sel:ident:$cur:ident;$callback:block;) => (
        {
            declare_machine!(@inner context $sel $cur);
            $callback;
            Some(States::__SameState__)
        }
    );

    // If Event have no user-defined code and move machine to new state. Just return new state.
    (@inner command @$glob_context:ident@ $sel:ident:$cur:ident; ;$new_state:ident$({$($new_el:ident:$new_el_val:expr),*})*) => (
        {
            declare_machine!(@inner context $sel $cur);
            $cur.leave($glob_context).unwrap();
            Some(States::$new_state{context: declare_machine!(@inner next $new_state$({$($new_el:$new_el_val),*})*)})
        }
    );

    // If Event have nothing to do on event. Just return __SameState__.
    (@inner command @$glob_context:ident@ $sel:ident:$cur:ident ; ;) => (
        Some(States::__SameState__)
    );

    (@inner context $ss:ident $sel:ident)=>(let $sel = $ss;);
    (@inner context $ss:ident )=>();

    // Enter/Leave processors with and without user-defined code.
    (@inner >> $($sel:ident)* @$glob_context:ident@ $income:block) => (
            fn enter(&mut self, $glob_context: &mut MachineContext) -> Result<(), ()> {
                declare_machine!(@inner context self $($sel)*);
                $income
                Ok(())
            }
    );
    (@inner << $($sel:ident)* @$glob_context:ident@ $outcome:block) => (
            fn leave(&mut self, $glob_context: &mut MachineContext) -> Result<(), ()> {
                declare_machine!(@inner context self $($sel)*);
                $outcome
                Ok(())
            }
    );
    (@inner >> $($sel:ident)* @$glob_context:ident@ ) => (
            fn enter(&mut self, $glob_context: &mut MachineContext) -> Result<(), ()> {
                Ok(())
            }
    );
    (@inner << $($sel:ident)* @$glob_context:ident@ ) => (
            fn leave(&mut self, $glob_context: &mut MachineContext) -> Result<(), ()> {
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

    (@cmd_processor $sel:ident @$glob_context:ident@ ($($cmd:ident $($callback:block)* => $($new_state:ident$({$($new_el:ident:$new_el_val:expr),*})*)*;)*))=>(
        fn do_job(&mut self, cmd: & Commands, $glob_context: &mut MachineContext) -> Option<States> {
            match *cmd {
                $(Commands::$cmd => {declare_machine!(@inner command @$glob_context@ self:$sel;$($callback)*;$($new_state$({$($new_el:$new_el_val),*})*)*)})*
                _ => None
            }
        }
    );

    (@state $gc_name:ident; $($state:ident @ $sel:ident ; $($income:block)*; ($job:tt); $($outcome:block)*@),*) => (
        $(
        impl CanDoJob for $state {
            declare_machine!(@cmd_processor $sel @$gc_name@ $job);
            declare_machine!(@inner >> $sel @$gc_name@ $($income)*);
            declare_machine!(@inner << $sel @$gc_name@ $($outcome)*);
        }
        )*
    );
    (@state ; $($state:ident @ $sel:ident ; $($income:block)*; ($job:tt); $($outcome:block)* @),*) => (
        $(
        impl CanDoJob for $state {
            declare_machine!(@cmd_processor $sel @__@ $job);
            declare_machine!(@inner >> $sel @__@ $($income)*);
            declare_machine!(@inner << $sel @__@ $($outcome)*);
        }
        )*
    );

    (@state $gc_name:ident; $($state:ident@; $($income:block)*; ($job:tt); $($outcome:block)*@),*) => (
        $(
        impl CanDoJob for $state {
            declare_machine!(@cmd_processor ___ @$gc_name@ $job);
            declare_machine!(@inner >> ___ @$gc_name@ $($income)*);
            declare_machine!(@inner << ___ @$gc_name@ $($outcome)*);
        }
        )*
    );
    (@state ; $($state:ident@; $($income:block)*; ($job:tt); $($outcome:block)*@),*) => (
        $(
        impl CanDoJob for $state {
            declare_machine!(@cmd_processor ___ @__@ $job);
            declare_machine!(@inner >> ___ @__@ $($income)*);
            declare_machine!(@inner << ___ @__@ $($outcome)*);
        }
        )*
    );

// Main pattern

(
    $machine:ident $($gc_name:ident{$($context_field:ident:$context_type:ty),*})* ($initial:ident$({$($init_field:ident:$init_val:expr),*})*)
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
            fn do_job(&mut self, cmd: &Commands, global_context: &mut MachineContext) -> Option<States>;
            fn leave(&mut self, &mut MachineContext) -> Result<(), ()>;
            fn enter(&mut self, &mut MachineContext) -> Result<(), ()>;
        }

        $(
        declare_machine!(@inner params $state $({$($el:$typ);*})*);
        )*

        declare_machine!(@state $($gc_name)*;$($state @ $($sel)* ; $($income)*; (($($cmd $($callback)* => $($new_state $({$($new_el:$new_el_val),*})*)*;)*)); $($outcome)*@),*);

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

        #[derive(Clone)]
        pub struct MachineContext {$($(pub $context_field: $context_type),*)*}

        pub struct Machine {
            state: States,
            context: MachineContext
        }
        pub fn new($($($context_field: $context_type),*)*) -> Machine {
            let mut context = declare_machine!(@inner initial $initial $({$($init_field: $init_val),*})*);
            let mut machine_context = MachineContext{$($($context_field: $context_field),*)*};
            context.enter(&mut machine_context).unwrap();
            Machine{state: States::$initial{context: context}, context: machine_context}
        }

        impl Machine {
            pub fn execute(&mut self, cmd: & Commands) -> Result<(),()>{
                match {
                    match self.state {
                        States::__SameState__ => None,
                        $(States::$state{ ref mut context } => context.do_job(cmd, &mut self.context)),*
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
                self.state = new_state;
                match self.state {
                    States::__SameState__ => Ok(()),
                    $(States::$state{ ref mut context } => context.enter(&mut self.context)),*
                }.unwrap();
            }
            pub fn state(&self) -> States {
                self.state.clone()
            }
            pub fn get_inner_context(&self) -> MachineContext {
                self.context.clone()
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

    declare_machine!(
    Mach3 glob_cont{id:i16}(State1{counter:0})

    states[State1,State2,State3]
    commands[ToState1, ToState2, ToState3]

    ( State1 cont{counter:i16}:
        >>{println!("Mach {} enter {:?}", glob_cont.id, cont);}
        <<{cont.counter+=1; println!("Mach {} leave {:?}", glob_cont.id, cont);}
        ToState2 => State2{counter: cont.counter};
    )
    ( State2 cont{counter:i16}:
        >>{println!("Mach {} enter {:?}", glob_cont.id, cont);}
        <<{cont.counter+=1; println!("Mach {} leave {:?}", glob_cont.id, cont);}
        ToState3 => State3{counter: cont.counter};
    )
    ( State3 cont{counter:i16}:
        >>{println!("Mach {} enter {:?}", glob_cont.id, cont);}
        <<{cont.counter+=1; println!("Mach {} leave {:?}", glob_cont.id, cont);}
        ToState1 => State1{counter: cont.counter};
    )
    );

    #[test]
    fn test3() {
        let mut m = Mach3::new(0);
        let mut m1 = Mach3::new(1);
        m1.execute(&Mach3::Commands::ToState2).unwrap();
        m.execute(&Mach3::Commands::ToState2).unwrap();
        m.execute(&Mach3::Commands::ToState3).unwrap();
        m1.execute(&Mach3::Commands::ToState3).unwrap();
    }

    #[derive(Clone)]
    pub struct InnerMachineContext {
        id: i16,
        name: String,
        counter: i16
    }

    declare_machine!(
        Mach4 inner{st: InnerMachineContext} (State1)
        states[State1,State2,State3]
        commands[ToState1, ToState2, ToState3]

        ( State1 :
            << {println!("id={} name={} counter={}", inner.st.id, inner.st.name, inner.st.counter);}
            ToState2 {inner.st.counter+=1;}=> State2;
        )
        ( State2 :
            << {println!("id={} name={} counter={}", inner.st.id, inner.st.name, inner.st.counter);}
            ToState3 {inner.st.counter+=1;}=> State3;
        )
        ( State3 :
            << {println!("id={} name={} counter={}", inner.st.id, inner.st.name, inner.st.counter);}
            ToState1 {inner.st.counter+=1;}=> State1;
        )
    );
    #[test]
    fn test4() {
        let mut m = Mach4::new(InnerMachineContext{id:0, name: String::from("Mach 0"), counter: 0});
        let mut m1 = Mach4::new(InnerMachineContext{id:1, name: String::from("Mach 1"), counter: 0});
        let mut m2 = Mach4::new(InnerMachineContext{id:2, name: String::from("Mach 2"), counter: 0});
        m.execute(&Mach4::Commands::ToState2).unwrap();
        m.execute(&Mach4::Commands::ToState3).unwrap();
        m1.execute(&Mach4::Commands::ToState2).unwrap();
        m.execute(&Mach4::Commands::ToState1).unwrap();
        m1.execute(&Mach4::Commands::ToState3).unwrap();
        m2.execute(&Mach4::Commands::ToState2).unwrap();
        m.execute(&Mach4::Commands::ToState2).unwrap();
        m2.execute(&Mach4::Commands::ToState3).unwrap();
        m1.execute(&Mach4::Commands::ToState1).unwrap();
    }
}
