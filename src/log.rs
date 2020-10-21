use clap::crate_name;

pub fn error<S: AsRef<str>>(msg: S) {
    eprintln!("{}: error: {}", crate_name!(), msg.as_ref());
}
