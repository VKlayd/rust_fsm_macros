macro_rules! FSM {
(
    name: $machine:ident (init: $initial:ident );
    states:($($states:ident),*);
    commands:($($commands:ident),*);

    $(($state:ident:
        $({$cmd:ident;$callback:stmt;$new_state:ident}),*
    )),*
) => (
    mod $machine {
        #[derive(Debug)]
        pub enum States {
            $($states),*
        }

        #[derive(Debug)]
        pub enum Commands {
            $($commands),*
        }

        pub struct Machine {
            state: States
        }
        pub fn new() -> Machine {
            Machine {state: States::$initial}
        }

        trait CanDoJob {
            fn do_job(cmd: & Commands) -> Option<States>;
        }

        $(
        struct $state{}
        impl CanDoJob for $state {
            fn do_job(cmd: & Commands) -> Option<States> {
                match *cmd {
                    $(Commands::$cmd => {
                        $callback;
                        Some(States::$new_state)
                    })*
                    _ => None
                }
            }
        }
        )*

        impl Machine {
            pub fn execute(&mut self, cmd: & Commands) {
                match {
                    match self.state {
                        $(States::$states => $states::do_job(cmd)),*
                    }
                } {
                    Some(x) => {println!("{:?}: {:?} -> {:?}", cmd, self.state, x); self.state = x},
                    None => println!("Wrong operation {:?} for {:?} state!", cmd, self.state)
                }
            }
        }
    }
)
}

#[cfg(test)]
mod tests {

    FSM!(
    name:Mach1(init:New);
    states:(New, InConfig, Operational);
    commands:(Configure, ConfigureDone);

    ( New:
        {Configure;     {println!("In New. Cmd Configure. Send InConfig.");};             InConfig},
        {ConfigureDone; {println!("In New. Cmd ConfigureDone. Send InConfig.");};         New}
    ),
    ( InConfig:
        {ConfigureDone; {println!("In InConfig. Cmd ConfigureDone. Send Operational.");}; Operational}
    ),
    ( Operational:

    )
    );

    #[test]
    fn test1() {
        let mut m = Mach1::new();
        m.execute(&Mach1::Commands::Configure);
        m.execute(&Mach1::Commands::ConfigureDone);
        m.execute(&Mach1::Commands::Configure);
    }
}
