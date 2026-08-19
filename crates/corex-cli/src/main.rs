//! `CorexPM` CLI bootstrap.

use corex_config::{resolve_config, LinkerMode, ScriptPolicy};
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
  -V, --version        Print version
  --json               Print machine-readable JSON output
  --linker <mode>      Set linker mode (isolated, virtual)
  --scripts <policy>   Set script lifecycle policy (deny, prompt, allow)
  --offline            Run in offline mode";

#[derive(serde::Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
enum CliOutput<T> {
    Success { data: T },
    Error { error: Diagnostic },
}

struct ParsedArgs {
    command: Option<String>,
    json: bool,
    linker: Option<LinkerMode>,
    scripts: Option<ScriptPolicy>,
    offline: Option<bool>,
    help: bool,
    version: bool,
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let parsed = match parse_args(args.into_iter()) {
        Ok(p) => p,
        Err(diag) => {
            let use_json = env::args().any(|a| a == "--json");
            print_error(diag, use_json);
            return ExitCode::from(1);
        }
    };

    let use_json = parsed.json;

    match execute(parsed) {
        Ok(out_json) => {
            if use_json {
                if let Some(json_val) = out_json {
                    println!("{json_val}");
                }
            }
            ExitCode::SUCCESS
        }
        Err(diag) => {
            print_error(diag.clone(), use_json);
            let code = if diag.code().starts_with("CXCLI") {
                1
            } else if diag.code().starts_with("CXRESOLVE") {
                2
            } else {
                1
            };
            ExitCode::from(code)
        }
    }
}

fn print_error(diag: Diagnostic, use_json: bool) {
    if use_json {
        let output = CliOutput::<()>::Error { error: diag };
        if let Ok(json_str) = serde_json::to_string_pretty(&output) {
            println!("{json_str}");
        } else {
            eprintln!("failed to serialize error to JSON");
        }
    } else {
        eprintln!("{diag}");
    }
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<ParsedArgs, Diagnostic> {
    let mut command = None;
    let mut json = false;
    let mut linker = None;
    let mut scripts = None;
    let mut offline = None;
    let mut help = false;
    let mut version = false;

    let mut iter = args.peekable();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--json" => {
                json = true;
            }
            "--offline" => {
                offline = Some(true);
            }
            "--no-offline" => {
                offline = Some(false);
            }
            "--linker" => {
                let val = iter.next().ok_or_else(|| {
                    Diagnostic::new(ErrorFamily::Cli, 1, "missing value for `--linker` option")
                        .with_help("supported values: isolated, virtual")
                })?;
                match val.to_lowercase().as_str() {
                    "isolated" => linker = Some(LinkerMode::Isolated),
                    "virtual" => linker = Some(LinkerMode::Virtual),
                    other => {
                        return Err(Diagnostic::new(
                            ErrorFamily::Cli,
                            1,
                            format!("invalid linker mode: `{other}`"),
                        )
                        .with_help("supported values: isolated, virtual"))
                    }
                }
            }
            "--scripts" => {
                let val = iter.next().ok_or_else(|| {
                    Diagnostic::new(ErrorFamily::Cli, 1, "missing value for `--scripts` option")
                        .with_help("supported values: deny, prompt, allow")
                })?;
                match val.to_lowercase().as_str() {
                    "deny" => scripts = Some(ScriptPolicy::Deny),
                    "prompt" => scripts = Some(ScriptPolicy::Prompt),
                    "allow" => scripts = Some(ScriptPolicy::Allow),
                    other => {
                        return Err(Diagnostic::new(
                            ErrorFamily::Cli,
                            1,
                            format!("invalid script policy: `{other}`"),
                        )
                        .with_help("supported values: deny, prompt, allow"))
                    }
                }
            }
            "-h" | "--help" | "help" => {
                help = true;
            }
            "-V" | "--version" => {
                version = true;
            }
            other if other.starts_with('-') => {
                return Err(Diagnostic::new(
                    ErrorFamily::Cli,
                    1,
                    format!("unknown option: `{other}`"),
                )
                .with_help("use `corexpm --help` to see available commands and options"));
            }
            other => {
                if command.is_some() {
                    return Err(Diagnostic::new(
                        ErrorFamily::Cli,
                        1,
                        format!("unexpected argument: `{other}`"),
                    )
                    .with_help("corexpm only accepts a single command at a time"));
                }
                command = Some(other.to_string());
            }
        }
    }

    Ok(ParsedArgs {
        command,
        json,
        linker,
        scripts,
        offline,
        help,
        version,
    })
}

fn execute(parsed: ParsedArgs) -> Result<Option<String>, Diagnostic> {
    let ParsedArgs {
        command,
        json,
        linker,
        scripts,
        offline,
        help,
        version,
    } = parsed;

    let config = resolve_config(None, linker, scripts, offline)?;
    let _ = corex_core::CorexContext { config };

    if version {
        if json {
            let data = serde_json::json!({
                "version": format!("{}-dev", corex_core::VERSION)
            });
            let output = CliOutput::Success { data };
            return Ok(Some(serde_json::to_string_pretty(&output).unwrap()));
        }
        println!("corexpm {}-dev", corex_core::VERSION);
        return Ok(None);
    }

    if help || command.is_none() {
        if json {
            let data = serde_json::json!({
                "help": HELP
            });
            let output = CliOutput::Success { data };
            return Ok(Some(serde_json::to_string_pretty(&output).unwrap()));
        }
        println!("{HELP}");
        return Ok(None);
    }

    let cmd = command.unwrap();
    match cmd.as_str() {
        "help" => {
            if json {
                let data = serde_json::json!({
                    "help": HELP
                });
                let output = CliOutput::Success { data };
                Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
            } else {
                println!("{HELP}");
                Ok(None)
            }
        }
        "doctor" => {
            let info = corex_core::doctor_info();
            if json {
                let output = CliOutput::Success { data: info };
                Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
            } else {
                println!("{info}");
                Ok(None)
            }
        }
        other => Err(Diagnostic::new(
            ErrorFamily::Cli,
            1,
            format!("command `{other}` is not implemented in the bootstrap release"),
        )
        .with_help("see docs/roadmap/ROADMAP.md for its target milestone")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args_options() {
        let args = vec![
            "doctor".to_owned(),
            "--json".to_owned(),
            "--linker".to_owned(),
            "virtual".to_owned(),
            "--scripts".to_owned(),
            "allow".to_owned(),
            "--offline".to_owned(),
        ];
        let parsed = parse_args(args.into_iter()).unwrap();
        assert_eq!(parsed.command.as_deref(), Some("doctor"));
        assert!(parsed.json);
        assert_eq!(parsed.linker, Some(LinkerMode::Virtual));
        assert_eq!(parsed.scripts, Some(ScriptPolicy::Allow));
        assert_eq!(parsed.offline, Some(true));
    }

    #[test]
    fn test_invalid_linker_option() {
        let args = vec!["--linker".to_owned(), "invalid".to_owned()];
        assert!(parse_args(args.into_iter()).is_err());
    }

    #[test]
    fn test_execute_doctor() {
        let parsed = ParsedArgs {
            command: Some("doctor".to_owned()),
            json: false,
            linker: None,
            scripts: None,
            offline: None,
            help: false,
            version: false,
        };
        let res = execute(parsed).unwrap();
        assert!(res.is_none());
    }

    #[test]
    fn test_execute_doctor_json() {
        let parsed = ParsedArgs {
            command: Some("doctor".to_owned()),
            json: true,
            linker: None,
            scripts: None,
            offline: None,
            help: false,
            version: false,
        };
        let res = execute(parsed).unwrap().unwrap();
        assert!(res.contains("\"status\": \"success\""));
        assert!(res.contains("\"version\""));
    }
}
