macro_rules! FSM {
    (@inner command $cur:ident;$callback:block;$new_state:ident{$($new_el:ident:$new_el_val:expr),*}) => (
        {
            $callback;
            Some(States::$new_state{context: $new_state{$($new_el:$new_el_val),*}})
        }
    );
    (@inner command $cur:ident;$callback:block;) => ({$callback; Some(States::__SameState__)});
    (@inner command $cur:ident; ;$new_state:ident{$($new_el:ident:$new_el_val:expr),*}) => ({
        Some(States::$new_state{context: $new_state{$($new_el:$new_el_val),*}})
    });
    (@inner command $cur:ident ; ;) => (Some(States::__SameState__));
(
    $machine:ident ($initial:ident{$($init_field:ident:$init_val:expr),*})
    commands:($($commands:ident),*)

    $(($state:ident $sel:ident{$($el:ident:$typ:ty),*}
        >> $income:block
        << $outcome:block
        $({$cmd:ident;$($callback:block)*;$($new_state:ident{$($new_el:ident:$new_el_val:expr),*})*})*
    ))*
) => (
    mod $machine {
        trait CanDoJob {
            fn do_job(&mut self, cmd: &Commands) -> Option<States>;
            fn leave(&self) -> Result<(), ()>;
            fn enter(&self) -> Result<(), ()>;
        }

        $(
        #[derive(Debug)]
        #[derive(PartialEq)]
        struct $state{$($el:$typ),*}

        impl CanDoJob for $state {
            fn do_job(&mut self, cmd: & Commands) -> Option<States> {
                let $sel = self;
                match *cmd {
                    $(Commands::$cmd => {FSM!(@inner command self;$($callback)*;$($new_state{$($new_el:$new_el_val),*})*)})*
                    _ => None
                }
            }
            fn leave(&self) -> Result<(), ()> {
                let $sel = self;
                $outcome
                Ok(())
            }
            fn enter(&self) -> Result<(), ()> {
                let $sel = self;
                $income
                Ok(())
            }
        }
        )*


        #[derive(Debug)]
        #[derive(PartialEq)]
        enum States {
            __SameState__,
            $($state {context: $state}),*
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
            let context = $initial{$($init_field: $init_val),*};
            context.enter().unwrap();
            Machine{state: States::$initial{context: context}}
        }

        impl Machine {
            pub fn execute(&mut self, cmd: & Commands) {
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
                        }
                    },
                    None => println!("Wrong operation {:?} for {:?} state!", cmd, self.state)
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
        }
    }
)
}

#[cfg(test)]
mod tests {

    FSM!(
    Mach1 (New{x:0})

    commands:(Configure, ConfigureDone, Drop)

    ( New context{x: i16}
        >> {println!("Enter {:?}", context)}
        << {println!("Leave {:?}", context)}
        {Configure;     {println!("In New with context val: {}", context.x);};     InConfig{x:context.x+1, y:0}}
        {ConfigureDone; ; New{x:0}}
    )
    ( InConfig context{x:i16, y:i16}
        >> {println!("Enter {:?}", context)}
        << {println!("Leave {:?}", context)}
        {ConfigureDone; ; Operational{x:context.x+1, y:context.y+1}}
    )
    ( Operational context{x:i16, y:i16}
        >> {println!("Enter {:?}", context)}
        << {println!("Leave {:?}", context)}
        {ConfigureDone; ; }
        {Drop; ; New{x:context.x+1}}
    )
    );

    #[test]
    fn test1() {
        let mut m = Mach1::new();
        m.execute(&Mach1::Commands::Configure);
        m.execute(&Mach1::Commands::ConfigureDone);
        m.execute(&Mach1::Commands::Configure);
        m.execute(&Mach1::Commands::Drop);
        m.execute(&Mach1::Commands::Configure);
        m.execute(&Mach1::Commands::ConfigureDone);
        m.execute(&Mach1::Commands::ConfigureDone);
        m.execute(&Mach1::Commands::ConfigureDone);
    }
}
