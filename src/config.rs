use clap::ArgMatches;

#[derive(Clone)]
pub struct Config {
    pub arguments: bool,
    pub threads: bool,
    pub verbose: bool,
}

impl Config {
    pub fn from_args(args: &ArgMatches) -> Self {
        Self {
            arguments: args.is_present("arguments"),
            threads: args.is_present("threads"),
            verbose: args.is_present("verbose"),
        }
    }
}
