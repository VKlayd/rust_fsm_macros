#![deny(unused_must_use)]
#[macro_export]
macro_rules! FSM {
    (@inner next $new_state:ident{$($new_el:ident:$new_el_val:expr),*}) => (
        $new_state{$($new_el:$new_el_val),*}
    );
    (@inner next $new_state:ident) => (
        $new_state{}
    );

    (@inner command $cur:ident;$callback:block;$new_state:ident$({$($new_el:ident:$new_el_val:expr),*})*) => (
        {
            $callback;
            Some(States::$new_state{context: FSM!(@inner next $new_state$({$($new_el:$new_el_val),*})*)})
        }
    );

    (@inner command $cur:ident;$callback:block;) => (
        {
            $callback;
            Some(States::__SameState__)
        }
    );

    (@inner command $cur:ident; ;$new_state:ident$({$($new_el:ident:$new_el_val:expr),*})*) => (
        {
            Some(States::$new_state{context: FSM!(@inner next $new_state$({$($new_el:$new_el_val),*})*)})
        }
    );

    (@inner command $cur:ident ; ;) => (
        Some(States::__SameState__)
    );

    (@inner context $ss:ident $sel:ident)=>(let $sel = $ss;);
    (@inner context $ss:ident )=>();

    (@inner >> $($sel:ident)* $income:block) => (
            fn enter(&mut self) -> Result<(), ()> {
                FSM!(@inner context self $($sel)*);
                $income
                Ok(())
            }
    );
    (@inner << $($sel:ident)* $outcome:block) => (
            fn leave(&mut self) -> Result<(), ()> {
                FSM!(@inner context self $($sel)*);
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
        FSM!(@inner params $state $({$($el:$typ);*})*);

        impl CanDoJob for $state {
            fn do_job(&mut self, cmd: & Commands) -> Option<States> {
                FSM!(@inner context self $($sel)*);
                match *cmd {
                    $(Commands::$cmd => {FSM!(@inner command self;$($callback)*;$($new_state$({$($new_el:$new_el_val),*})*)*)})*
                    _ => None
                }
            }

            FSM!(@inner >> $($sel)* $($income)*);
            FSM!(@inner << $($sel)* $($outcome)*);
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
            let mut context = FSM!(@inner initial $initial $({$($init_field: $init_val),*})*);
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

    FSM!(
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

    FSM!(
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
