//! `CorexPM` CLI bootstrap.

use corex_config::{resolve_config, LinkerMode, ScriptPolicy};
use corex_errors::{Diagnostic, ErrorFamily};
use std::env;
use std::process::ExitCode;

const HELP: &str = "CorexPM — secure, disk-efficient JavaScript package management

Usage: corexpm <COMMAND> [OPTIONS]

Bootstrap and package commands:
  doctor              Report the local development environment
  help                 Print this help
  info                 Show package registry information
  install, i           Resolve project dependencies
  why                  Explain why a package is present
  store                Inspect or maintain Corex CAS

Planned package commands:
  init                 Create a package manifest and Corex configuration
  add                  Add a dependency
  remove               Remove a dependency
  update               Update dependencies within declared ranges
  list                 Show installed dependencies

Planned execution and security commands:
  run                  Run a package script
  exec                 Run a package binary
  audit                Inspect known vulnerabilities and policy violations
  permissions          Inspect dependency capabilities

Planned workspace and storage commands:
  workspace            Inspect or operate on workspace packages
  changed              List packages changed from a revision
  cache                Inspect or maintain registry/download caches

Options:
  -h, --help           Print help
  -V, --version        Print version
  --json               Print machine-readable JSON output
  --linker <mode>      Set linker mode (isolated, virtual)
  --scripts <policy>   Set script lifecycle policy (deny, prompt, allow)
  --offline            Run in offline mode
  --fixtures <dir>     Directory containing local registry JSON fixtures";

#[derive(serde::Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
enum CliOutput<T> {
    Success { data: T },
    Error { error: Diagnostic },
}

#[allow(clippy::struct_excessive_bools)]
struct ParsedArgs {
    command: Option<String>,
    command_args: Vec<String>,
    json: bool,
    linker: Option<LinkerMode>,
    scripts: Option<ScriptPolicy>,
    offline: Option<bool>,
    help: bool,
    version: bool,
    fixtures: Option<String>,
    target_workspaces: Vec<String>,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    all: bool,
    changed: bool,
    affected: bool,
    concurrency: Option<usize>,
    fail_fast: Option<bool>,
    min_severity: Option<corex_audit::VulnerabilitySeverity>,
    ignore_advisories: Vec<String>,
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

#[allow(clippy::too_many_lines)]
fn parse_args(args: impl Iterator<Item = String>) -> Result<ParsedArgs, Diagnostic> {
    let mut command = None;
    let mut command_args = Vec::new();
    let mut json = false;
    let mut linker = None;
    let mut scripts = None;
    let mut offline = None;
    let mut help = false;
    let mut version = false;
    let mut fixtures = None;
    let mut target_workspaces = Vec::new();
    let mut include_patterns = Vec::new();
    let mut exclude_patterns = Vec::new();
    let mut all = false;
    let mut changed = false;
    let mut affected = false;
    let mut concurrency = None;
    let mut fail_fast = None;
    let mut min_severity = None;
    let mut ignore_advisories = Vec::new();

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
            "--all" => {
                all = true;
            }
            "--changed" => {
                changed = true;
            }
            "--affected" => {
                affected = true;
            }
            "--fail-fast" => {
                fail_fast = Some(true);
            }
            "--no-fail-fast" => {
                fail_fast = Some(false);
            }
            "-w" | "--workspace" => {
                let val = iter.next().ok_or_else(|| {
                    Diagnostic::new(
                        ErrorFamily::Cli,
                        1,
                        "missing value for `--workspace` option",
                    )
                })?;
                target_workspaces.push(val);
            }
            "--include" => {
                let val = iter.next().ok_or_else(|| {
                    Diagnostic::new(ErrorFamily::Cli, 1, "missing value for `--include` option")
                })?;
                include_patterns.push(val);
            }
            "--exclude" => {
                let val = iter.next().ok_or_else(|| {
                    Diagnostic::new(ErrorFamily::Cli, 1, "missing value for `--exclude` option")
                })?;
                exclude_patterns.push(val);
            }
            "--concurrency" => {
                let val = iter.next().ok_or_else(|| {
                    Diagnostic::new(
                        ErrorFamily::Cli,
                        1,
                        "missing value for `--concurrency` option",
                    )
                })?;
                let count = val.parse::<usize>().map_err(|_| {
                    Diagnostic::new(
                        ErrorFamily::Cli,
                        1,
                        format!("invalid concurrency value `{val}`"),
                    )
                })?;
                concurrency = Some(count);
            }
            "--severity" => {
                let val = iter.next().ok_or_else(|| {
                    Diagnostic::new(ErrorFamily::Cli, 1, "missing value for `--severity` option")
                        .with_help("supported values: low, medium, high, critical")
                })?;
                match val.to_lowercase().as_str() {
                    "low" => min_severity = Some(corex_audit::VulnerabilitySeverity::Low),
                    "medium" => min_severity = Some(corex_audit::VulnerabilitySeverity::Medium),
                    "high" => min_severity = Some(corex_audit::VulnerabilitySeverity::High),
                    "critical" => min_severity = Some(corex_audit::VulnerabilitySeverity::Critical),
                    other => {
                        return Err(Diagnostic::new(
                            ErrorFamily::Cli,
                            1,
                            format!("invalid severity level: `{other}`"),
                        )
                        .with_help("supported values: low, medium, high, critical"))
                    }
                }
            }
            "--ignore" => {
                let val = iter.next().ok_or_else(|| {
                    Diagnostic::new(ErrorFamily::Cli, 1, "missing value for `--ignore` option")
                })?;
                ignore_advisories.push(val);
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
            "--fixtures" => {
                let val = iter.next().ok_or_else(|| {
                    Diagnostic::new(ErrorFamily::Cli, 1, "missing value for `--fixtures` option")
                })?;
                fixtures = Some(val);
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
                if command.is_none() {
                    command = Some(other.to_string());
                } else {
                    command_args.push(other.to_string());
                }
            }
        }
    }

    Ok(ParsedArgs {
        command,
        command_args,
        json,
        linker,
        scripts,
        offline,
        help,
        version,
        fixtures,
        target_workspaces,
        include_patterns,
        exclude_patterns,
        all,
        changed,
        affected,
        concurrency,
        fail_fast,
        min_severity,
        ignore_advisories,
    })
}

fn default_fixtures_dir() -> std::path::PathBuf {
    std::env::current_dir().map_or_else(
        |_| std::path::PathBuf::from("tests/fixtures/registry"),
        |cwd| {
            let mut current = cwd.as_path();
            loop {
                let fixtures = current.join("tests").join("fixtures").join("registry");
                if fixtures.exists() {
                    return fixtures;
                }
                if let Some(parent) = current.parent() {
                    current = parent;
                } else {
                    break;
                }
            }
            cwd.join("tests").join("fixtures").join("registry")
        },
    )
}

fn find_paths(
    target: corex_graph::NodeId,
    graph: &corex_graph::DependencyGraph,
    current_path: &mut Vec<corex_graph::NodeId>,
    paths: &mut Vec<Vec<corex_graph::NodeId>>,
) {
    if graph.root_nodes.contains(&target) {
        let mut full_path = vec![target];
        full_path.extend(current_path.iter().rev());
        paths.push(full_path);
    }

    for edge in &graph.edges {
        if edge.to == target && !current_path.contains(&edge.from) {
            current_path.push(target);
            find_paths(edge.from, graph, current_path, paths);
            current_path.pop();
        }
    }
}

#[allow(clippy::too_many_lines)]
fn execute(parsed: ParsedArgs) -> Result<Option<String>, Diagnostic> {
    let ParsedArgs {
        command,
        command_args,
        json,
        linker,
        scripts,
        offline,
        help,
        version,
        fixtures,
        target_workspaces,
        include_patterns,
        exclude_patterns,
        all,
        changed,
        affected,
        concurrency,
        fail_fast,
        min_severity,
        ignore_advisories,
    } = parsed;

    let config = resolve_config(None, linker, scripts, offline)?;
    let context = corex_core::CorexContext { config };

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
        "info" => {
            let pkg_arg = command_args.first().ok_or_else(|| {
                Diagnostic::new(
                    ErrorFamily::Cli,
                    1,
                    "missing package name for `info` command",
                )
                .with_help("Usage: corexpm info <package-name>")
            })?;

            let fixtures_path =
                fixtures.map_or_else(default_fixtures_dir, std::path::PathBuf::from);
            let info = corex_core::fetch_package_info(pkg_arg, &fixtures_path)?;

            if json {
                let output = CliOutput::Success { data: info };
                Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
            } else {
                println!("Package:           {}", info.name.as_str());
                println!(
                    "Latest version:    {}",
                    info.dist_tags.get("latest").cloned().unwrap_or_default()
                );
                let mut vers: Vec<_> = info
                    .versions
                    .keys()
                    .map(corex_semver::Version::as_str)
                    .collect();
                vers.sort();
                println!("Versions:          {}", vers.join(", "));
                Ok(None)
            }
        }
        "install" | "i" => {
            let is_frozen = command_args
                .iter()
                .any(|a| a == "--frozen" || a == "--frozen-lockfile" || a == "--immutable");
            let fixtures_path =
                fixtures.map_or_else(default_fixtures_dir, std::path::PathBuf::from);

            let project_root = std::env::current_dir().map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Cli,
                    2,
                    format!("failed to read current working directory: {e}"),
                )
            })?;

            let result = if is_frozen {
                corex_core::install_project_frozen(&project_root, &context, &fixtures_path)?
            } else {
                corex_core::install_project(&project_root, &context, &fixtures_path)?
            };

            if json {
                let output = CliOutput::Success { data: result };
                Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
            } else {
                if result.reconciled {
                    println!(
                        "Reconciled `{}` in {}ms (up to date)",
                        result.manifest_name, result.elapsed_ms
                    );
                } else {
                    println!(
                        "Installed `{}` in {}ms (frozen: {is_frozen})",
                        result.manifest_name, result.elapsed_ms
                    );
                    println!("  Resolved packages:     {}", result.resolved_count);
                    println!(
                        "  Direct dependencies:   {}",
                        result.summary.direct_dependencies
                    );
                    println!(
                        "  Virtual instances:     {}",
                        result.summary.virtual_instances
                    );
                    println!("  Symlinks created:      {}", result.summary.total_links);
                    println!("  Binary shims linked:   {}", result.summary.binary_links);
                }
                Ok(None)
            }
        }
        "ci" => {
            let fixtures_path =
                fixtures.map_or_else(default_fixtures_dir, std::path::PathBuf::from);

            let project_root = std::env::current_dir().map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Cli,
                    2,
                    format!("failed to read current working directory: {e}"),
                )
            })?;

            let result =
                corex_core::install_project_frozen(&project_root, &context, &fixtures_path)?;

            if json {
                let output = CliOutput::Success { data: result };
                Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
            } else {
                println!(
                    "CI Frozen Install completed for `{}` in {}ms",
                    result.manifest_name, result.elapsed_ms
                );
                println!("  Resolved packages: {}", result.resolved_count);
                Ok(None)
            }
        }
        "lockfile" | "lock" => {
            let sub = command_args.first().map_or("verify", String::as_str);
            let project_root = std::env::current_dir().map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Cli,
                    2,
                    format!("failed to read current working directory: {e}"),
                )
            })?;
            let lockfile_path = project_root.join("corex.lock.json");

            match sub {
                "path" => {
                    if json {
                        let output = CliOutput::Success {
                            data: serde_json::json!({ "path": lockfile_path }),
                        };
                        Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
                    } else {
                        println!("{}", lockfile_path.display());
                        Ok(None)
                    }
                }
                "verify" | "status" => {
                    let lockfile = corex_core::verify_lockfile(&project_root)?;
                    if json {
                        let output = CliOutput::Success { data: lockfile };
                        Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
                    } else {
                        println!("Lockfile `corex.lock.json` verified cleanly!");
                        println!("  Version:   {}", lockfile.lockfile_version);
                        println!("  Packages:  {}", lockfile.packages.len());
                        Ok(None)
                    }
                }
                _ => Err(Diagnostic::new(
                    ErrorFamily::Cli,
                    3,
                    format!("unknown lockfile subcommand `{sub}`"),
                )
                .with_help("supported lockfile subcommands: path, verify, status")),
            }
        }
        "add" => {
            let pkg_arg = command_args.first().ok_or_else(|| {
                Diagnostic::new(
                    ErrorFamily::Cli,
                    1,
                    "missing package name for `add` command",
                )
                .with_help("Usage: corexpm add <package-name> [--dev]")
            })?;

            let is_dev = command_args.iter().any(|a| a == "--dev" || a == "-D");
            let fixtures_path =
                fixtures.map_or_else(default_fixtures_dir, std::path::PathBuf::from);

            let project_root = std::env::current_dir().map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Cli,
                    2,
                    format!("failed to read current working directory: {e}"),
                )
            })?;

            let result = corex_core::add_dependency(
                &project_root,
                &context,
                &fixtures_path,
                pkg_arg,
                None,
                is_dev,
            )?;

            if json {
                let output = CliOutput::Success { data: result };
                Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
            } else {
                println!(
                    "Added `{pkg_arg}` and installed project in {}ms",
                    result.elapsed_ms
                );
                Ok(None)
            }
        }
        "remove" => {
            let pkg_arg = command_args.first().ok_or_else(|| {
                Diagnostic::new(
                    ErrorFamily::Cli,
                    1,
                    "missing package name for `remove` command",
                )
                .with_help("Usage: corexpm remove <package-name>")
            })?;

            let fixtures_path =
                fixtures.map_or_else(default_fixtures_dir, std::path::PathBuf::from);

            let project_root = std::env::current_dir().map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Cli,
                    2,
                    format!("failed to read current working directory: {e}"),
                )
            })?;

            let result =
                corex_core::remove_dependency(&project_root, &context, &fixtures_path, pkg_arg)?;

            if json {
                let output = CliOutput::Success { data: result };
                Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
            } else {
                println!(
                    "Removed `{pkg_arg}` and reconciled project in {}ms",
                    result.elapsed_ms
                );
                Ok(None)
            }
        }
        "list" => {
            let fixtures_path =
                fixtures.map_or_else(default_fixtures_dir, std::path::PathBuf::from);

            let project_root = std::env::current_dir().map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Cli,
                    2,
                    format!("failed to read current working directory: {e}"),
                )
            })?;

            let tree = corex_core::list_dependencies(&project_root, &context, &fixtures_path)?;

            if json {
                let output = CliOutput::Success { data: tree };
                Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
            } else {
                println!("{}", serde_json::to_string_pretty(&tree).unwrap());
                Ok(None)
            }
        }
        "trust" => {
            let sub = command_args.first().map_or("list", String::as_str);
            let target_pkg = command_args.get(1);

            let project_root = std::env::current_dir().map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Cli,
                    2,
                    format!("failed to read current working directory: {e}"),
                )
            })?;

            match sub {
                "approve" => {
                    let pkg = target_pkg.ok_or_else(|| {
                        Diagnostic::new(
                            ErrorFamily::Cli,
                            1,
                            "missing package name for `trust approve`",
                        )
                        .with_help("Usage: corexpm trust approve <package-name>")
                    })?;
                    corex_core::approve_trust(&project_root, pkg)?;
                    if json {
                        let output = CliOutput::Success {
                            data: serde_json::json!({ "approved": pkg }),
                        };
                        Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
                    } else {
                        println!("Approved dependency script execution for `{pkg}`");
                        Ok(None)
                    }
                }
                "deny" => {
                    let pkg = target_pkg.ok_or_else(|| {
                        Diagnostic::new(
                            ErrorFamily::Cli,
                            1,
                            "missing package name for `trust deny`",
                        )
                        .with_help("Usage: corexpm trust deny <package-name>")
                    })?;
                    corex_core::deny_trust(&project_root, pkg)?;
                    if json {
                        let output = CliOutput::Success {
                            data: serde_json::json!({ "denied": pkg }),
                        };
                        Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
                    } else {
                        println!("Denied dependency script execution for `{pkg}`");
                        Ok(None)
                    }
                }
                "list" => {
                    let store = corex_core::list_trust(&project_root)?;
                    if json {
                        let output = CliOutput::Success { data: store };
                        Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
                    } else {
                        println!("Trusted Packages:");
                        for (pkg, decision) in &store.packages {
                            println!("  {pkg}: {decision:?}");
                        }
                        Ok(None)
                    }
                }
                _ => Err(Diagnostic::new(
                    ErrorFamily::Cli,
                    3,
                    format!("unknown trust subcommand `{sub}`"),
                )
                .with_help("supported trust subcommands: approve, deny, list")),
            }
        }
        "audit" => {
            let fixtures_path =
                fixtures.map_or_else(default_fixtures_dir, std::path::PathBuf::from);

            let project_root = std::env::current_dir().map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Cli,
                    2,
                    format!("failed to read current working directory: {e}"),
                )
            })?;

            let summary = corex_core::audit_project(
                &project_root,
                &context,
                &fixtures_path,
                min_severity,
                &ignore_advisories,
            )?;

            if json {
                let output = CliOutput::Success { data: summary };
                Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
            } else {
                println!(
                    "Security Audit Summary: {} packages scanned, {} vulnerabilities found",
                    summary.scanned_packages, summary.vulnerabilities_found
                );
                println!(
                    "  Critical: {} | High: {} | Medium: {} | Low: {}",
                    summary.critical_count,
                    summary.high_count,
                    summary.medium_count,
                    summary.low_count
                );

                if !summary.matches.is_empty() {
                    println!("\nVulnerabilities:");
                    for m in &summary.matches {
                        println!(
                            "  [{}] {} v{} — {}",
                            m.advisory.severity.as_str().to_uppercase(),
                            m.package_name,
                            m.installed_version,
                            m.advisory.title
                        );
                        println!(
                            "    Advisory ID: {} | Vulnerable range: {}",
                            m.advisory.id, m.advisory.vulnerable_range
                        );
                        if let Some(patched) = &m.advisory.patched_version {
                            println!("    Patched version: {patched}");
                        }
                    }
                }
                Ok(None)
            }
        }
        "permissions" => {
            let fixtures_path =
                fixtures.map_or_else(default_fixtures_dir, std::path::PathBuf::from);

            let project_root = std::env::current_dir().map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Cli,
                    2,
                    format!("failed to read current working directory: {e}"),
                )
            })?;

            let report =
                corex_core::get_permissions_report(&project_root, &context, &fixtures_path)?;
            let enforcement = corex_core::evaluate_platform_security();

            if json {
                let data = serde_json::json!({
                    "permissions": report,
                    "platform_enforcement": enforcement,
                });
                let output = CliOutput::Success { data };
                Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
            } else {
                println!("Corex Guard Effective Policy:");
                for entry in &report.entries {
                    println!(
                        "  {:20} => {:?} ({})",
                        entry.package_name, entry.effective_action, entry.policy_source
                    );
                    println!("    {}", entry.explanation);
                }

                println!(
                    "\nPlatform Capability Enforcement ({} / {}):",
                    enforcement.os, enforcement.arch
                );
                for cap in &enforcement.capabilities {
                    println!(
                        "  {:30} => [{}] {}",
                        cap.key,
                        cap.level.as_str().to_uppercase(),
                        cap.description
                    );
                }
                Ok(None)
            }
        }
        "workspace" | "ws" => {
            let project_root = std::env::current_dir().map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Cli,
                    2,
                    format!("failed to read current working directory: {e}"),
                )
            })?;

            let metadata = corex_core::list_workspace_members(&project_root)?;

            if json {
                let output = CliOutput::Success { data: metadata };
                Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
            } else {
                println!("Workspace Root: {}", metadata.root_dir.display());
                println!("Members ({}):", metadata.packages.len());
                for (name, pkg) in &metadata.packages {
                    println!("  {} ({})", name, pkg.relative_path.display());
                }
                Ok(None)
            }
        }
        "migrate" | "import" => {
            let project_root = std::env::current_dir().map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Cli,
                    2,
                    format!("failed to read current working directory: {e}"),
                )
            })?;

            let summary = corex_core::migrate_lockfile(&project_root)?;

            if json {
                let output = CliOutput::Success { data: summary };
                Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
            } else {
                println!(
                    "Successfully imported {} lockfile ({}) to `corex.lock.json` ({} packages).",
                    summary.format,
                    summary.source_path.display(),
                    summary.packages_migrated
                );
                println!("Invariant verified: Original foreign lockfile was preserved untouched.");
                Ok(None)
            }
        }
        "changed" => {
            let project_root = std::env::current_dir().map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Cli,
                    2,
                    format!("failed to read current working directory: {e}"),
                )
            })?;

            let changed_set = corex_core::list_changed_packages(&project_root, None)?;

            if json {
                let output = CliOutput::Success { data: changed_set };
                Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
            } else {
                if changed_set.is_empty() {
                    println!("No packages changed.");
                } else {
                    for pkg in &changed_set {
                        println!("{pkg}");
                    }
                }
                Ok(None)
            }
        }
        "run" => {
            let script_arg = command_args.first().ok_or_else(|| {
                Diagnostic::new(ErrorFamily::Cli, 1, "missing script name for `run` command")
                    .with_help("Usage: corexpm run <script-name>")
            })?;

            let project_root = std::env::current_dir().map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Cli,
                    2,
                    format!("failed to read current working directory: {e}"),
                )
            })?;

            let is_ws_mode = all
                || changed
                || affected
                || !target_workspaces.is_empty()
                || !include_patterns.is_empty()
                || !exclude_patterns.is_empty()
                || corex_core::list_workspace_members(&project_root)
                    .is_ok_and(|m| !m.packages.is_empty());

            if is_ws_mode {
                let filter = corex_workspace::WorkspaceFilter {
                    target_workspaces,
                    include_patterns,
                    exclude_patterns,
                    all,
                };
                let options = corex_workspace::TaskSchedulerOptions {
                    concurrency: concurrency.unwrap_or(4),
                    fail_fast: fail_fast.unwrap_or(true),
                };

                let summary = corex_core::run_workspace_script(
                    &project_root,
                    script_arg,
                    &filter,
                    &options,
                    changed,
                    affected,
                )?;

                if json {
                    let output = CliOutput::Success { data: summary };
                    Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
                } else {
                    println!(
                        "Executed workspace script `{script_arg}` across {} packages ({} succeeded, {} failed, {} skipped)",
                        summary.total_tasks, summary.successful_tasks, summary.failed_tasks, summary.skipped_tasks
                    );
                    for (pkg, res) in &summary.results {
                        let status = if res.success { "success" } else { "failed" };
                        println!("  {pkg}: {status}");
                    }
                    Ok(None)
                }
            } else {
                let result = corex_core::run_project_script(&project_root, script_arg)?;

                if json {
                    let output = CliOutput::Success { data: result };
                    Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
                } else {
                    println!(
                        "Executed script `{script_arg}` in {}ms (success: {})",
                        result.duration_ms, result.success
                    );
                    if !result.stdout.is_empty() {
                        println!("{}", result.stdout);
                    }
                    if !result.stderr.is_empty() {
                        eprintln!("{}", result.stderr);
                    }
                    Ok(None)
                }
            }
        }
        "exec" => {
            let bin_arg = command_args.first().ok_or_else(|| {
                Diagnostic::new(
                    ErrorFamily::Cli,
                    1,
                    "missing binary name for `exec` command",
                )
                .with_help("Usage: corexpm exec <binary-name>")
            })?;

            let project_root = std::env::current_dir().map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Cli,
                    2,
                    format!("failed to read current working directory: {e}"),
                )
            })?;

            let bin_path = corex_core::exec_binary(&project_root, bin_arg)?;

            if json {
                let output = CliOutput::Success {
                    data: serde_json::json!({
                        "binary": bin_arg,
                        "path": bin_path,
                    }),
                };
                Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
            } else {
                println!("Executable binary `{bin_arg}` at: {}", bin_path.display());
                Ok(None)
            }
        }
        "why" => {
            let pkg_arg = command_args.first().ok_or_else(|| {
                Diagnostic::new(
                    ErrorFamily::Cli,
                    1,
                    "missing package name for `why` command",
                )
                .with_help("Usage: corexpm why <package-name>")
            })?;

            let fixtures_path =
                fixtures.map_or_else(default_fixtures_dir, std::path::PathBuf::from);

            let project_root = std::env::current_dir().map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Cli,
                    2,
                    format!("failed to read current working directory: {e}"),
                )
            })?;
            let manifest_path = project_root.join("package.json");
            if !manifest_path.exists() {
                return Err(Diagnostic::new(
                    ErrorFamily::Resolve,
                    1,
                    "package.json not found in the current directory",
                )
                .with_help("run `corexpm init` or create a package.json manually"));
            }

            let manifest_content = std::fs::read_to_string(&manifest_path).map_err(|e| {
                Diagnostic::new(
                    ErrorFamily::Resolve,
                    2,
                    format!(
                        "failed to read package.json at `{}`: {e}",
                        manifest_path.display()
                    ),
                )
            })?;

            let graph = corex_core::resolve_manifest(&manifest_content, &context, &fixtures_path)?;

            let mut paths = Vec::new();
            for node in graph.nodes.values() {
                if node.package.name().as_str() == pkg_arg {
                    let mut path = Vec::new();
                    find_paths(node.id, &graph, &mut path, &mut paths);
                }
            }

            if json {
                let output = CliOutput::Success { data: paths };
                Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
            } else {
                if paths.is_empty() {
                    println!(
                        "Package `{pkg_arg}` is not present in the resolved dependency graph."
                    );
                } else {
                    println!("Package `{pkg_arg}` is present because:");
                    for path in &paths {
                        let path_str = path
                            .iter()
                            .map(|&id| {
                                let node = graph.nodes.get(&id).unwrap();
                                format!(
                                    "{}@{}",
                                    node.package.name().as_str(),
                                    node.version.version()
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(" -> ");
                        println!("  {path_str}");
                    }
                }
                Ok(None)
            }
        }
        "store" => {
            let subcmd = command_args.first().map_or("status", String::as_str);
            match subcmd {
                "path" => {
                    let path = corex_core::store_path(None);
                    if json {
                        let output = CliOutput::Success {
                            data: serde_json::json!({ "path": path }),
                        };
                        Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
                    } else {
                        println!("{}", path.display());
                        Ok(None)
                    }
                }
                "status" | "stats" => {
                    let stats = corex_core::store_stats(None)?;
                    if json {
                        let output = CliOutput::Success { data: stats };
                        Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
                    } else {
                        println!("Store path:         {}", stats.store_path.display());
                        println!("Unique packages:    {}", stats.package_count);
                        println!("Physical size:      {} bytes", stats.physical_bytes);
                        println!("Logical referenced: {} bytes", stats.logical_bytes);
                        println!("Deduplicated saved: {} bytes", stats.saved_bytes);
                        println!("Registered projects:{}", stats.project_count);
                        Ok(None)
                    }
                }
                "verify" | "validate" => {
                    let report = corex_core::store_verify(None)?;
                    if json {
                        let output = CliOutput::Success { data: report };
                        Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
                    } else {
                        println!("Store Verification Summary:");
                        println!("  Valid package objects:    {}", report.valid_count);
                        println!("  Corrupt package objects:  {}", report.corrupt_count);
                        for detail in &report.corrupt_details {
                            println!("    - {detail}");
                        }
                        Ok(None)
                    }
                }
                "prune" => {
                    let mut grace_period = 86400u64; // default 24 hours
                    let mut iter = command_args.iter().skip(1);
                    while let Some(arg) = iter.next() {
                        if arg == "--grace-period" {
                            if let Some(val) = iter.next() {
                                grace_period = val.parse::<u64>().map_err(|_| {
                                    Diagnostic::new(
                                        ErrorFamily::Cli,
                                        1,
                                        format!("invalid grace period value: `{val}`"),
                                    )
                                })?;
                            }
                        } else if let Ok(val) = arg.parse::<u64>() {
                            grace_period = val;
                        }
                    }

                    let summary = corex_core::store_prune(None, grace_period)?;
                    if json {
                        let output = CliOutput::Success { data: summary };
                        Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
                    } else {
                        println!("Store Prune Summary:");
                        println!("  Removed packages:  {}", summary.removed_count);
                        println!("  Reclaimed space:   {} bytes", summary.reclaimed_bytes);
                        for key in &summary.pruned_keys {
                            println!("    - Pruned: {key}");
                        }
                        Ok(None)
                    }
                }
                other => Err(Diagnostic::new(
                    ErrorFamily::Cli,
                    1,
                    format!("unknown subcommand for `store`: `{other}`"),
                )
                .with_help("supported subcommands: path, status, stats, verify, prune")),
            }
        }
        "cache" => {
            let subcmd = command_args.first().map_or("status", String::as_str);
            match subcmd {
                "path" => {
                    let path = corex_core::cache_path(None);
                    if json {
                        let output = CliOutput::Success {
                            data: serde_json::json!({ "path": path }),
                        };
                        Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
                    } else {
                        println!("{}", path.display());
                        Ok(None)
                    }
                }
                "status" => {
                    let mode = if context.config.offline {
                        corex_cache::CacheMode::Offline
                    } else {
                        corex_cache::CacheMode::Online
                    };
                    let status = corex_core::cache_status(None, mode)?;
                    if json {
                        let output = CliOutput::Success { data: status };
                        Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
                    } else {
                        println!("Cache path:        {}", status.path.display());
                        println!("Cached metadata:   {} files", status.metadata_count);
                        println!("Cached tarballs:   {} files", status.tarball_count);
                        println!("Total size:        {} bytes", status.total_bytes);
                        Ok(None)
                    }
                }
                "clean" => {
                    corex_core::cache_clean(None)?;
                    if json {
                        let output = CliOutput::Success {
                            data: serde_json::json!({ "cleaned": true }),
                        };
                        Ok(Some(serde_json::to_string_pretty(&output).unwrap()))
                    } else {
                        println!("Successfully cleaned local cache.");
                        Ok(None)
                    }
                }
                other => Err(Diagnostic::new(
                    ErrorFamily::Cli,
                    1,
                    format!("unknown subcommand for `cache`: `{other}`"),
                )
                .with_help("supported subcommands: path, status, clean")),
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
            "info".to_owned(),
            "react".to_owned(),
            "--json".to_owned(),
            "--linker".to_owned(),
            "virtual".to_owned(),
            "--fixtures".to_owned(),
            "/tmp/mock".to_owned(),
        ];
        let parsed = parse_args(args.into_iter()).unwrap();
        assert_eq!(parsed.command.as_deref(), Some("info"));
        assert_eq!(parsed.command_args, vec!["react".to_owned()]);
        assert!(parsed.json);
        assert_eq!(parsed.linker, Some(LinkerMode::Virtual));
        assert_eq!(parsed.fixtures.as_deref(), Some("/tmp/mock"));
    }

    #[test]
    fn test_invalid_linker_option() {
        let args = vec!["--linker".to_owned(), "invalid".to_owned()];
        assert!(parse_args(args.into_iter()).is_err());
    }

    fn default_test_parsed_args(cmd: &str) -> ParsedArgs {
        ParsedArgs {
            command: Some(cmd.to_owned()),
            command_args: Vec::new(),
            json: false,
            linker: None,
            scripts: None,
            offline: None,
            help: false,
            version: false,
            fixtures: None,
            target_workspaces: Vec::new(),
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            all: false,
            changed: false,
            affected: false,
            concurrency: None,
            fail_fast: None,
            min_severity: None,
            ignore_advisories: Vec::new(),
        }
    }

    #[test]
    fn test_execute_doctor() {
        let parsed = default_test_parsed_args("doctor");
        let res = execute(parsed).unwrap();
        assert!(res.is_none());
    }

    #[test]
    fn test_execute_doctor_json() {
        let mut parsed = default_test_parsed_args("doctor");
        parsed.json = true;
        let res = execute(parsed).unwrap().unwrap();
        assert!(res.contains("\"status\": \"success\""));
        assert!(res.contains("\"version\""));
    }

    #[test]
    fn test_execute_store_commands() {
        let mut parsed = default_test_parsed_args("store");
        parsed.command_args = vec!["status".to_owned()];
        parsed.json = true;
        let res = execute(parsed).unwrap().unwrap();
        assert!(res.contains("\"package_count\""));
    }

    #[test]
    fn test_execute_cache_commands() {
        let mut parsed = default_test_parsed_args("cache");
        parsed.command_args = vec!["status".to_owned()];
        parsed.json = true;
        let res = execute(parsed).unwrap().unwrap();
        assert!(res.contains("\"metadata_count\""));
    }

    #[test]
    fn test_parse_args_workspace_options() {
        let args = vec![
            "run".to_owned(),
            "build".to_owned(),
            "-w".to_owned(),
            "@app/web".to_owned(),
            "--all".to_owned(),
            "--concurrency".to_owned(),
            "8".to_owned(),
            "--no-fail-fast".to_owned(),
        ];
        let parsed = parse_args(args.into_iter()).unwrap();
        assert_eq!(parsed.command.as_deref(), Some("run"));
        assert_eq!(parsed.command_args, vec!["build".to_owned()]);
        assert_eq!(parsed.target_workspaces, vec!["@app/web".to_owned()]);
        assert_eq!(parsed.concurrency, Some(8));
        assert_eq!(parsed.fail_fast, Some(false));
    }

    #[test]
    fn test_parse_args_audit_options() {
        let args = vec![
            "audit".to_owned(),
            "--severity".to_owned(),
            "high".to_owned(),
            "--ignore".to_owned(),
            "CX-ADV-2026-001".to_owned(),
            "--json".to_owned(),
        ];
        let parsed = parse_args(args.into_iter()).unwrap();
        assert_eq!(parsed.command.as_deref(), Some("audit"));
        assert_eq!(
            parsed.min_severity,
            Some(corex_audit::VulnerabilitySeverity::High)
        );
        assert_eq!(parsed.ignore_advisories, vec!["CX-ADV-2026-001".to_owned()]);
        assert!(parsed.json);
    }

    #[test]
    fn test_execute_migrate_command() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("package-lock.json"),
            r#"{ "name": "test", "version": "1.0.0", "dependencies": { "express": { "version": "4.18.2" } } }"#,
        )
        .unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();

        let parsed = ParsedArgs {
            command: Some("migrate".to_string()),
            json: true,
            ..default_test_parsed_args("migrate")
        };

        let result = execute(parsed).unwrap();
        assert!(result.is_some());
        assert!(tmp.path().join("corex.lock.json").exists());
        assert!(tmp.path().join("package-lock.json").exists());

        std::env::set_current_dir(original_dir).unwrap();
    }
}
