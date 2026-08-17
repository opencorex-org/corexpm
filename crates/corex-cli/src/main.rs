//! `CorexPM` CLI bootstrap.

use corex_errors::{Diagnostic, ErrorFamily};
use std::env;
use std::process::ExitCode;

const HELP: &str = "CorexPM — secure, disk-efficient JavaScript package management

Usage: corexpm <COMMAND> [OPTIONS]

Bootstrap commands:
  doctor              Report the local development environment
  help                 Print this help

Planned package commands:
  init                 Create a package manifest and Corex configuration
  install, i           Resolve and install project dependencies
  add                  Add a dependency
  remove               Remove a dependency
  update               Update dependencies within declared ranges
  list                 Show installed dependencies
  why                  Explain why a package is present

Planned execution and security commands:
  run                  Run a package script
  exec                 Run a package binary
  audit                Inspect known vulnerabilities and policy violations
  permissions          Inspect dependency capabilities

Planned workspace and storage commands:
  workspace            Inspect or operate on workspace packages
  changed              List packages changed from a revision
  store                Inspect or maintain Corex CAS
  cache                Inspect or maintain registry/download caches

Options:
  -h, --help           Print help
  -V, --version        Print version";

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(mut args: impl Iterator<Item = String>) -> Result<(), Diagnostic> {
    let command = args.next();
    match command.as_deref() {
        None | Some("help" | "-h" | "--help") => {
            println!("{HELP}");
            Ok(())
        }
        Some("-V" | "--version") => {
            println!("corexpm {}-dev", corex_core::VERSION);
            Ok(())
        }
        Some("doctor") => {
            println!("{}", corex_core::doctor_report());
            Ok(())
        }
        Some(command) => Err(Diagnostic::new(
            ErrorFamily::Cli,
            1,
            format!("command `{command}` is not implemented in the bootstrap release"),
        )
        .with_help("see docs/roadmap/ROADMAP.md for its target milestone")),
    }
}

#[cfg(test)]
mod tests {
    use super::run;

    #[test]
    fn version_is_available() {
        assert!(run(["--version".to_owned()].into_iter()).is_ok());
    }

    #[test]
    fn planned_command_is_explicitly_rejected() {
        let error = run(["install".to_owned()].into_iter()).unwrap_err();
        assert_eq!(error.code(), "CXCLI0001");
    }
}
