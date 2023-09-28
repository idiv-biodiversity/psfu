use std::path::{Path, PathBuf};
use std::{env, fs, io};

use clap_complete::Shell;

#[path = "src/pid.rs"]
mod pid;

#[allow(dead_code)]
#[path = "src/cli.rs"]
mod cli;

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-env-changed=DIST_DIR");
    println!("cargo:rerun-if-env-changed=PROFILE");

    println!("cargo:rerun-if-changed=src/cli.rs");

    let profile = env::var_os("PROFILE");
    if profile != Some("release".into()) {
        println!(
            "cargo:warning=\
             generating shell completions only in release mode, \
             current: PROFILE={}",
            profile.unwrap_or_default().to_string_lossy()
        );
    } else {
        let name = env::var("CARGO_PKG_NAME")
            .ok()
            .ok_or(io::ErrorKind::NotFound)?;

        let destination = env::var_os("DIST_DIR")
            .or_else(|| env::var_os("OUT_DIR"))
            .ok_or(io::ErrorKind::NotFound)?;

        let destination = PathBuf::from(destination);
        if !destination.exists() {
            fs::create_dir_all(&destination)?;
        }

        println!(
            "cargo:warning=\
             generating shell completions to {}",
            destination.display()
        );

        gen_completion(&name, &destination, Shell::Bash)?;
        gen_completion(&name, &destination, Shell::Fish)?;
        gen_completion(&name, &destination, Shell::Elvish)?;
        gen_completion(&name, &destination, Shell::Zsh)?;
    }

    Ok(())
}

fn gen_completion(
    name: impl AsRef<str>,
    destination: impl AsRef<Path>,
    shell: Shell,
) -> io::Result<()> {
    clap_complete::generate_to(
        shell,
        &mut cli::build(),
        name.as_ref(),
        destination.as_ref(),
    )
    .map(|_| ())
}
