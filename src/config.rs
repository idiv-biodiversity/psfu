use clap::ArgMatches;

#[derive(Clone)]
pub struct Config {
    pub threads: bool,
    pub verbose: bool,
}

impl Config {
    pub fn from_args(args: &ArgMatches) -> Config {
        Config {
            threads: args.is_present("threads"),
            verbose: args.is_present("verbose"),
        }
    }
}
