use atty::Stream;
use clap::{crate_description, crate_name, crate_version};
use clap::{App, AppSettings, Arg, SubCommand};

pub fn build() -> App<'static, 'static> {
    let color = if atty::is(Stream::Stdout) {
        AppSettings::ColoredHelp
    } else {
        AppSettings::ColorNever
    };

    // ------------------------------------------------------------------------
    // arguments
    // ------------------------------------------------------------------------

    let pid = Arg::with_name("pid")
        .help("process IDs")
        .multiple(true)
        .required(atty::is(Stream::Stdin))
        .validator(is_pid);

    let arguments = Arg::with_name("arguments")
        .long("arguments")
        .short("a")
        .help("show arguments");

    let cpuset = Arg::with_name("cpuset")
        .help("cpuset")
        .required(true)
        .validator(is_cpuset);

    let threads = Arg::with_name("threads")
        .long("threads")
        .short("t")
        .help("include threads");

    let verbose = Arg::with_name("verbose")
        .long("verbose")
        .long_help("Adds verbose output.")
        .hidden_short_help(true);

    // ------------------------------------------------------------------------
    // show commands
    // ------------------------------------------------------------------------

    let plain = SubCommand::with_name("plain")
        .arg(&pid)
        .arg(&arguments)
        .arg(&threads)
        .about("show process tree")
        .help_short("?")
        .help_message("show this help output");

    let affinity_s = SubCommand::with_name("affinity")
        .arg(&pid)
        .arg(&threads)
        .about("show process tree with affinity (cpuset)")
        .help_short("?")
        .help_message("show this help output");

    let backtrace = SubCommand::with_name("backtrace")
        .alias("bt")
        .arg(&pid)
        .arg(&threads)
        .arg(&verbose)
        .about("show process tree with backtrace")
        .help_short("?")
        .help_message("show this help output");

    // ------------------------------------------------------------------------
    // modify commands
    // ------------------------------------------------------------------------

    let affinity_m = SubCommand::with_name("affinity")
        .arg(&cpuset)
        .arg(&pid)
        .arg(&threads)
        .arg(&verbose)
        .about("modify process tree affinity (cpuset)")
        .help_short("?")
        .help_message("show this help output");

    // ------------------------------------------------------------------------
    // tree commands
    // ------------------------------------------------------------------------

    let modify = SubCommand::with_name("modify")
        .about("modify processes")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(affinity_m)
        .help_short("?")
        .help_message("show this help output");

    let show = SubCommand::with_name("show")
        .about("show processes")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(affinity_s)
        .subcommand(backtrace)
        .subcommand(plain)
        .help_short("?")
        .help_message("show this help output");

    // ------------------------------------------------------------------------
    // top-level commands
    // ------------------------------------------------------------------------

    let tree = SubCommand::with_name("tree")
        .about("process tree commands")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(modify)
        .subcommand(show)
        .help_short("?")
        .help_message("show this help output");

    // ------------------------------------------------------------------------
    // put it all together
    // ------------------------------------------------------------------------

    App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .global_setting(color)
        .global_setting(AppSettings::ArgsNegateSubcommands)
        .global_setting(AppSettings::InferSubcommands)
        .global_setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(tree)
        .help_short("?")
        .help_message("show this help output")
        .version_message("show version")
}

#[allow(clippy::needless_pass_by_value)]
fn is_cpuset(s: String) -> Result<(), String> {
    if s == "free" || s.parse::<u64>().is_ok() {
        Ok(())
    } else {
        Err(format!("invalid cpuset: {:?}", s))
    }
}

#[allow(clippy::needless_pass_by_value)]
fn is_pid(s: String) -> Result<(), String> {
    crate::pid::validate(s).map(|_| ())
}
