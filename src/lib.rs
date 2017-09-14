macro_rules! FSM {
(
    name: $machine:ident (init: $initial:ident{$($init_field:ident:$init_val:expr),*});
    states:($($states:ident),*);
    commands:($($commands:ident),*);

    $(($state:ident {$($el:ident:$typ:ty),*} /$sel:ident/:
        >> $income:block,
        << $outcome:block,
        $({$cmd:ident;$callback:block;$new_state:ident{$($new_el:ident:$new_el_val:expr),*}}),*
    )),*
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
                    $(Commands::$cmd => {
                        $callback;
                        Some(States::$new_state{context: $new_state{$($new_el:$new_el_val),*}})
                    })*
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
        pub enum States {
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
            let context = $initial{$($init_field: $init_val),*};
            context.enter().unwrap();
            Machine{state: States::$initial{context: context}}
        }

        impl Machine {
            pub fn execute(&mut self, cmd: & Commands) {
                match {
                    match self.state {
                        $(States::$states{ ref mut context } => context.do_job(cmd)),*
                    }
                } {
                    Some(x) => {
                        if x != self.state {
                            self.change_state(x)
                        }
                    },
                    None => println!("Wrong operation {:?} for {:?} state!", cmd, self.state)
                }
            }
            fn change_state(&mut self, new_state: States) {
                match self.state {
                    $(States::$states{ ref mut context } => context.leave()),*
                }.unwrap();
                self.state = new_state;
                match self.state {
                    $(States::$states{ ref mut context } => context.enter()),*
                }.unwrap();
            }
        }
    }
)
}

#[cfg(test)]
mod tests {

    FSM!(
    name:Mach1 (init: New{x:0});
    states:(New, InConfig, Operational);
    commands:(Configure, ConfigureDone, Drop);

    ( New {x: i16} /context/:
        >> {println!("Enter {:?}", context)},
        << {println!("Leave {:?}", context)},
        {Configure;     {println!("In New with context val: {}", context.x);};     InConfig{x:context.x+1, y:0}},
        {ConfigureDone; {};   New{x:0}}
    ),
    ( InConfig {x:i16, y:i16} /context/:
        >> {println!("Enter {:?}", context)},
        << {println!("Leave {:?}", context)},
        {ConfigureDone; {}; Operational{x:context.x+1, y:context.y+1}}
    ),
    ( Operational {x:i16, y:i16} /context/:
        >> {println!("Enter {:?}", context)},
        << {println!("Leave {:?}", context)},
        {Drop; {}; New{x:context.x+1}}
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
    }
}
