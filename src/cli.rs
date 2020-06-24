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
        .help("process ID")
        .required(true)
        .validator(is_pid);

    let threads = Arg::with_name("threads")
        .long("threads")
        .help("include threads");

    let verbose = Arg::with_name("verbose")
        .long("verbose")
        .long_help("Adds verbose output.")
        .hidden_short_help(true);

    // ------------------------------------------------------------------------
    // tree commands
    // ------------------------------------------------------------------------

    let backtrace = SubCommand::with_name("backtrace")
        .alias("bt")
        .arg(&pid)
        .arg(&threads)
        .arg(&verbose)
        .about("run backtrace over process tree")
        .help_short("?")
        .help_message("show this help output");

    let show = SubCommand::with_name("show")
        .arg(&pid)
        .arg(&threads)
        .about("show process tree")
        .help_short("?")
        .help_message("show this help output");

    // ------------------------------------------------------------------------
    // top-level commands
    // ------------------------------------------------------------------------

    let tree = SubCommand::with_name("tree")
        .about("process tree commands")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(backtrace)
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

fn is_pid(s: String) -> Result<(), String> {
    s.parse::<u64>()
        .map_err(|e| format!("{} is not a process ID: {}", s, e))
        .and_then(check_pid_max)
}

fn check_pid_max(pid: u64) -> Result<(), String> {
    let pid_max = std::fs::read_to_string("/proc/sys/kernel/pid_max")
        .map_err(|e| format!("reading pid_max: {}", e))?
        .trim()
        .parse::<u64>()
        .map_err(|e| format!("parsing pid_max: {}", e))?;

    if pid <= pid_max {
        Ok(())
    } else {
        Err(format!(
            "process ID {} is higher than pid_max {}",
            pid, pid_max
        ))
    }
}
