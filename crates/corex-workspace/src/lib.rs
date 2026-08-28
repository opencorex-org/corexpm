//! `CorexPM` workspace discovery, graph intelligence, filtering, and bounded task scheduling.

pub mod changed;
pub mod discovery;
pub mod filter;
pub mod graph;
pub mod scheduler;

pub use changed::WorkspaceChanged;
pub use discovery::{WorkspaceDiscovery, WorkspaceMetadata, WorkspacePackage};
pub use filter::WorkspaceFilter;
pub use graph::{WorkspaceGraph, WorkspaceNode};
pub use scheduler::{TaskExecutionSummary, TaskResult, TaskScheduler, TaskSchedulerOptions};

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn create_temp_dir() -> PathBuf {
        let mut dir = std::env::temp_dir();
        let num = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        dir.push(format!("corex_ws_test_{num}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_workspace_discovery_and_graph_waves() {
        let root = create_temp_dir();

        let root_manifest = r#"{
            "name": "root",
            "workspaces": ["packages/*"]
        }"#;
        fs::write(root.join("package.json"), root_manifest).unwrap();

        let dir_alpha = root.join("packages/pkg-a");
        let dir_beta = root.join("packages/pkg-b");
        fs::create_dir_all(&dir_alpha).unwrap();
        fs::create_dir_all(&dir_beta).unwrap();

        let manifest_a = r#"{
            "name": "pkg-a",
            "scripts": { "build": "echo building-a" }
        }"#;
        let manifest_b = r#"{
            "name": "pkg-b",
            "dependencies": { "pkg-a": "workspace:*" },
            "scripts": { "build": "echo building-b" }
        }"#;

        fs::write(dir_alpha.join("package.json"), manifest_a).unwrap();
        fs::write(dir_beta.join("package.json"), manifest_b).unwrap();

        let discovery = WorkspaceDiscovery::new();
        let metadata = discovery.discover(&root).unwrap();

        assert_eq!(metadata.packages.len(), 2);
        assert!(metadata.packages.contains_key("pkg-a"));
        assert!(metadata.packages.contains_key("pkg-b"));

        let graph = WorkspaceGraph::build(&metadata).unwrap();
        assert_eq!(graph.execution_waves.len(), 2);
        assert_eq!(graph.execution_waves[0], vec!["pkg-a"]);
        assert_eq!(graph.execution_waves[1], vec!["pkg-b"]);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_workspace_cycle_detection() {
        let root = create_temp_dir();

        let root_manifest = r#"{
            "name": "root",
            "workspaces": ["packages/*"]
        }"#;
        fs::write(root.join("package.json"), root_manifest).unwrap();

        let dir_alpha = root.join("packages/pkg-a");
        let dir_beta = root.join("packages/pkg-b");
        fs::create_dir_all(&dir_alpha).unwrap();
        fs::create_dir_all(&dir_beta).unwrap();

        let manifest_a = r#"{
            "name": "pkg-a",
            "dependencies": { "pkg-b": "workspace:*" }
        }"#;
        let manifest_b = r#"{
            "name": "pkg-b",
            "dependencies": { "pkg-a": "workspace:*" }
        }"#;

        fs::write(dir_alpha.join("package.json"), manifest_a).unwrap();
        fs::write(dir_beta.join("package.json"), manifest_b).unwrap();

        let discovery = WorkspaceDiscovery::new();
        let metadata = discovery.discover(&root).unwrap();

        let res = WorkspaceGraph::build(&metadata);
        assert!(res.is_err());
        let diag = res.unwrap_err();
        assert_eq!(diag.code(), "CXWORK0001");
        assert!(diag.message().contains("cycle detected"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_workspace_changed_and_affected() {
        let root = create_temp_dir();

        let root_manifest = r#"{
            "name": "root",
            "workspaces": ["packages/*"]
        }"#;
        fs::write(root.join("package.json"), root_manifest).unwrap();

        let dir_alpha = root.join("packages/pkg-a");
        let dir_beta = root.join("packages/pkg-b");
        fs::create_dir_all(&dir_alpha).unwrap();
        fs::create_dir_all(&dir_beta).unwrap();

        fs::write(dir_alpha.join("package.json"), r#"{"name": "pkg-a"}"#).unwrap();
        fs::write(
            dir_beta.join("package.json"),
            r#"{"name": "pkg-b", "dependencies": {"pkg-a": "*"}}"#,
        )
        .unwrap();

        let discovery = WorkspaceDiscovery::new();
        let metadata = discovery.discover(&root).unwrap();
        let graph = WorkspaceGraph::build(&metadata).unwrap();

        let changed_calc = WorkspaceChanged::new();
        let changed_paths = vec![dir_alpha.join("src/index.ts")];

        let changed = changed_calc.calculate_changed(&graph, &root, &changed_paths);
        assert_eq!(changed.len(), 1);
        assert!(changed.contains("pkg-a"));

        let affected = changed_calc.calculate_affected(&graph, &changed);
        assert_eq!(affected.len(), 2);
        assert!(affected.contains("pkg-a"));
        assert!(affected.contains("pkg-b"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_workspace_task_scheduler() {
        let root = create_temp_dir();

        let root_manifest = r#"{
            "name": "root",
            "workspaces": ["packages/*"]
        }"#;
        fs::write(root.join("package.json"), root_manifest).unwrap();

        let dir_alpha = root.join("packages/pkg-a");
        let dir_beta = root.join("packages/pkg-b");
        fs::create_dir_all(&dir_alpha).unwrap();
        fs::create_dir_all(&dir_beta).unwrap();

        fs::write(
            dir_alpha.join("package.json"),
            r#"{"name": "pkg-a", "scripts": {"test": "echo test-a"}}"#,
        )
        .unwrap();
        fs::write(
            dir_beta.join("package.json"),
            r#"{"name": "pkg-b", "scripts": {"test": "echo test-b"}}"#,
        )
        .unwrap();

        let discovery = WorkspaceDiscovery::new();
        let metadata = discovery.discover(&root).unwrap();
        let graph = WorkspaceGraph::build(&metadata).unwrap();

        let filter = WorkspaceFilter {
            all: true,
            ..Default::default()
        };
        let target_packages = filter.select_packages(&graph, None).unwrap();

        let scheduler = TaskScheduler::new();
        let summary = scheduler
            .execute_script(
                &graph,
                &target_packages,
                "test",
                &TaskSchedulerOptions::default(),
            )
            .unwrap();

        assert_eq!(summary.total_tasks, 2);
        assert_eq!(summary.successful_tasks, 2);
        assert_eq!(summary.failed_tasks, 0);

        fs::remove_dir_all(root).unwrap();
    }
}
