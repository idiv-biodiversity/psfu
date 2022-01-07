use atty::Stream;
use clap::{crate_description, crate_name, crate_version};
use clap::{App, AppSettings, Arg};

pub fn build() -> App<'static> {
    // ------------------------------------------------------------------------
    // arguments
    // ------------------------------------------------------------------------

    let pid = Arg::new("pid")
        .help("process IDs")
        .multiple_values(true)
        .required(atty::is(Stream::Stdin))
        .validator(is_pid);

    let arguments = Arg::new("arguments")
        .long("arguments")
        .short('a')
        .help("show arguments");

    let cpuset = Arg::new("cpuset")
        .help("cpuset")
        .required(true)
        .validator(is_cpuset);

    let threads = Arg::new("threads")
        .long("threads")
        .short('t')
        .help("include threads");

    let verbose = Arg::new("verbose")
        .long("verbose")
        .long_help("Adds verbose output.")
        .hide_short_help(true);

    // ------------------------------------------------------------------------
    // show commands
    // ------------------------------------------------------------------------

    let plain = App::new("plain")
        .arg(&pid)
        .arg(&arguments)
        .arg(&threads)
        .about("show process tree")
        .mut_arg("help", |a| {
            a.short('?').help("print help").long_help("Print help.")
        });

    let affinity_s = App::new("affinity")
        .arg(&pid)
        .arg(&threads)
        .about("show process tree with affinity (cpuset)")
        .mut_arg("help", |a| {
            a.short('?').help("print help").long_help("Print help.")
        });

    let backtrace = App::new("backtrace")
        .alias("bt")
        .arg(&pid)
        .arg(&threads)
        .arg(&verbose)
        .about("show process tree with backtrace")
        .mut_arg("help", |a| {
            a.short('?').help("print help").long_help("Print help.")
        });

    // ------------------------------------------------------------------------
    // modify commands
    // ------------------------------------------------------------------------

    let affinity_m = App::new("affinity")
        .arg(&cpuset)
        .arg(&pid)
        .arg(&threads)
        .arg(&verbose)
        .about("modify process tree affinity (cpuset)")
        .mut_arg("help", |a| {
            a.short('?').help("print help").long_help("Print help.")
        });

    // ------------------------------------------------------------------------
    // tree commands
    // ------------------------------------------------------------------------

    let modify = App::new("modify")
        .about("modify processes")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(affinity_m)
        .mut_arg("help", |a| {
            a.short('?').help("print help").long_help("Print help.")
        });

    let show = App::new("show")
        .about("show processes")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(affinity_s)
        .subcommand(backtrace)
        .subcommand(plain)
        .mut_arg("help", |a| {
            a.short('?').help("print help").long_help("Print help.")
        });

    // ------------------------------------------------------------------------
    // top-level commands
    // ------------------------------------------------------------------------

    let tree = App::new("tree")
        .about("process tree commands")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(modify)
        .subcommand(show)
        .mut_arg("help", |a| {
            a.short('?').help("print help").long_help("Print help.")
        });

    // ------------------------------------------------------------------------
    // put it all together
    // ------------------------------------------------------------------------

    App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .global_setting(AppSettings::ArgsNegateSubcommands)
        .global_setting(AppSettings::InferSubcommands)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(tree)
        .mut_arg("help", |a| {
            a.short('?').help("print help").long_help("Print help.")
        })
}

fn is_cpuset(s: &str) -> Result<(), String> {
    if s == "free" || s.parse::<u64>().is_ok() {
        Ok(())
    } else {
        Err(format!("invalid cpuset: {:?}", s))
    }
}

fn is_pid(s: &str) -> Result<(), String> {
    crate::pid::validate(s).map(|_| ())
}

pub const SUBCOMMAND_REQUIRED: &str =
    "CLI argument parser should have been set up to require a subcommand";
