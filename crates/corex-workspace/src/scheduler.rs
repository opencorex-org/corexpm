//! Bounded-concurrency topological task scheduler.

use crate::graph::WorkspaceGraph;
use corex_errors::{Diagnostic, ErrorFamily};
use corex_scripts::ScriptExecutor;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use std::thread;

/// Per-package task execution result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskResult {
    /// Target package name.
    pub package_name: String,
    /// Executed script name.
    pub script_name: String,
    /// Command string executed.
    pub command: String,
    /// Whether task completed successfully with exit code 0.
    pub success: bool,
    /// Process exit status code.
    pub exit_code: i32,
    /// Redacted standard output.
    pub stdout: String,
    /// Redacted standard error.
    pub stderr: String,
}

/// Aggregated task execution summary across selected workspace packages.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskExecutionSummary {
    /// Total tasks evaluated.
    pub total_tasks: usize,
    /// Tasks completed successfully.
    pub successful_tasks: usize,
    /// Tasks that failed.
    pub failed_tasks: usize,
    /// Tasks skipped because script was missing.
    pub skipped_tasks: usize,
    /// Map of package name to execution result.
    pub results: BTreeMap<String, TaskResult>,
}

/// Task scheduler configuration and options.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskSchedulerOptions {
    /// Maximum concurrent worker threads.
    pub concurrency: usize,
    /// Whether to cancel pending tasks on the first failure (`fail-fast`).
    pub fail_fast: bool,
}

impl Default for TaskSchedulerOptions {
    fn default() -> Self {
        Self {
            concurrency: 4,
            fail_fast: true,
        }
    }
}

/// Topological bounded-concurrency task scheduler.
#[derive(Debug, Default)]
pub struct TaskScheduler;

impl TaskScheduler {
    /// Creates a new task scheduler.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Executes script `script_name` across `target_packages` in topological wave order.
    ///
    /// # Errors
    ///
    /// Returns a [`Diagnostic`] if `fail_fast` is true and a task fails or concurrency execution fails.
    pub fn execute_script(
        &self,
        graph: &WorkspaceGraph,
        target_packages: &BTreeSet<String>,
        script_name: &str,
        options: &TaskSchedulerOptions,
    ) -> Result<TaskExecutionSummary, Diagnostic> {
        let executor = ScriptExecutor::new();
        let mut summary = TaskExecutionSummary::default();

        // Process wave by wave to guarantee topological dependency ordering
        for wave in &graph.execution_waves {
            let wave_targets: Vec<String> = wave
                .iter()
                .filter(|name| target_packages.contains(*name))
                .cloned()
                .collect();

            if wave_targets.is_empty() {
                continue;
            }

            // Filter targets that actually define the requested script
            let mut executable_targets = Vec::new();
            for pkg_name in wave_targets {
                if let Some(node) = graph.nodes.get(&pkg_name) {
                    if let Some(cmd) = node.package.manifest.scripts.get(script_name) {
                        executable_targets.push((
                            pkg_name.clone(),
                            node.package.root_path.clone(),
                            cmd.clone(),
                        ));
                    } else {
                        summary.skipped_tasks += 1;
                    }
                }
            }

            if executable_targets.is_empty() {
                continue;
            }

            // Execute current wave with bounded concurrency pool
            let wave_results = execute_wave_parallel(
                &executable_targets,
                script_name,
                options.concurrency,
                &executor,
            )?;

            let mut wave_failed = false;
            for result in wave_results {
                summary.total_tasks += 1;
                if result.success {
                    summary.successful_tasks += 1;
                } else {
                    summary.failed_tasks += 1;
                    wave_failed = true;
                }
                summary.results.insert(result.package_name.clone(), result);
            }

            if wave_failed && options.fail_fast {
                return Err(Diagnostic::new(
                    ErrorFamily::Workspace,
                    3,
                    format!("workspace script `{script_name}` failed in wave execution"),
                )
                .with_help("rerun with --no-fail-fast to allow independent packages to continue"));
            }
        }

        Ok(summary)
    }
}

fn execute_wave_parallel(
    targets: &[(String, std::path::PathBuf, String)],
    script_name: &str,
    max_concurrency: usize,
    executor: &ScriptExecutor,
) -> Result<Vec<TaskResult>, Diagnostic> {
    let concurrency = std::cmp::max(1, max_concurrency);
    let targets = targets.to_vec();
    let script_name = script_name.to_string();

    let queue = Arc::new(Mutex::new(targets.into_iter().collect::<Vec<_>>()));
    let results = Arc::new(Mutex::new(Vec::new()));

    let worker_count = std::cmp::min(concurrency, queue.lock().unwrap().len());
    let mut handles = Vec::new();

    for _ in 0..worker_count {
        let queue_clone = Arc::clone(&queue);
        let results_clone = Arc::clone(&results);
        let script_name_clone = script_name.clone();
        let executor_clone = executor.clone();

        let handle = thread::spawn(move || loop {
            let item = {
                let mut q = queue_clone.lock().unwrap();
                q.pop()
            };

            let Some((pkg_name, root_path, cmd)) = item else {
                break;
            };

            let res = executor_clone.execute_script(
                &root_path,
                &root_path,
                &pkg_name,
                &script_name_clone,
                &cmd,
            );

            let task_res = match res {
                Ok(exec) => TaskResult {
                    package_name: pkg_name,
                    script_name: script_name_clone.clone(),
                    command: cmd,
                    success: exec.success,
                    exit_code: i32::from(!exec.success),
                    stdout: exec.stdout,
                    stderr: exec.stderr,
                },
                Err(diag) => TaskResult {
                    package_name: pkg_name,
                    script_name: script_name_clone.clone(),
                    command: cmd,
                    success: false,
                    exit_code: 1,
                    stdout: String::new(),
                    stderr: diag.message().to_string(),
                },
            };

            results_clone.lock().unwrap().push(task_res);
        });

        handles.push(handle);
    }

    for handle in handles {
        if handle.join().is_err() {
            return Err(Diagnostic::new(
                ErrorFamily::Workspace,
                3,
                "worker thread panicked during workspace task execution",
            ));
        }
    }

    let mut res = results.lock().unwrap().clone();
    res.sort_by(|a, b| a.package_name.cmp(&b.package_name));
    Ok(res)
}
