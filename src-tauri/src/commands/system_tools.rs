//! Local project tools for Git (GitHub Sync).
//!
//! These commands operate only against the active Catalyst project folder.
//! They do not make project content cloud-authoritative; Git remains optional
//! version control over the normal local files.

use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug, Serialize)]
pub struct GitWorkspaceStatus {
    pub root: String,
    pub is_repository: bool,
    pub branch: Option<String>,
    pub remote: Option<String>,
    pub changes: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommandResult {
    pub command: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

fn resolve_project_root(raw: &str) -> Result<PathBuf, String> {
    let root = PathBuf::from(raw.trim());
    if !root.is_absolute() {
        return Err("Project folder must be an absolute local path.".into());
    }
    if !root.is_dir() {
        return Err(format!("Project folder '{}' does not exist.", root.display()));
    }
    Ok(root)
}

fn git(root: &Path, args: &[&str]) -> Result<CommandResult, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| format!("Could not run Git: {e}"))?;
    Ok(CommandResult {
        command: format!("git {}", args.join(" ")),
        exit_code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

#[tauri::command]
pub async fn get_git_workspace_status(project_root: String) -> Result<GitWorkspaceStatus, String> {
    let root = resolve_project_root(&project_root)?;
    let probe = git(&root, &["rev-parse", "--is-inside-work-tree"])?;
    let is_repository = probe.exit_code == Some(0) && probe.stdout.trim() == "true";
    if !is_repository {
        return Ok(GitWorkspaceStatus {
            root: root.to_string_lossy().into_owned(),
            is_repository: false,
            branch: None,
            remote: None,
            changes: Vec::new(),
        });
    }

    let branch = git(&root, &["branch", "--show-current"])?.stdout.trim().to_string();
    let remote = git(&root, &["remote", "get-url", "origin"])
        .ok()
        .filter(|result| result.exit_code == Some(0))
        .map(|result| result.stdout.trim().to_string())
        .filter(|value| !value.is_empty());
    let changes = git(&root, &["status", "--short"])?
        .stdout
        .lines()
        .map(str::to_string)
        .collect();

    Ok(GitWorkspaceStatus {
        root: root.to_string_lossy().into_owned(),
        is_repository: true,
        branch: (!branch.is_empty()).then_some(branch),
        remote,
        changes,
    })
}

#[tauri::command]
pub async fn git_initialize(project_root: String) -> Result<CommandResult, String> {
    let root = resolve_project_root(&project_root)?;
    git(&root, &["init"])
}

#[tauri::command]
pub async fn git_set_origin(project_root: String, remote: String) -> Result<CommandResult, String> {
    let root = resolve_project_root(&project_root)?;
    let remote = remote.trim();
    if remote.is_empty() {
        return Err("Git remote URL must not be empty.".into());
    }
    let current = git(&root, &["remote"])?;
    if current.stdout.lines().any(|name| name.trim() == "origin") {
        git(&root, &["remote", "set-url", "origin", remote])
    } else {
        git(&root, &["remote", "add", "origin", remote])
    }
}

#[tauri::command]
pub async fn git_commit_all(project_root: String, message: String) -> Result<Vec<CommandResult>, String> {
    let root = resolve_project_root(&project_root)?;
    let message = message.trim();
    if message.is_empty() {
        return Err("Commit message must not be empty.".into());
    }
    let add = git(&root, &["add", "-A"])?;
    if add.exit_code != Some(0) {
        return Ok(vec![add]);
    }
    let commit = git(&root, &["commit", "-m", message])?;
    Ok(vec![add, commit])
}

#[tauri::command]
pub async fn git_pull(project_root: String) -> Result<CommandResult, String> {
    let root = resolve_project_root(&project_root)?;
    git(&root, &["pull"])
}

#[tauri::command]
pub async fn git_push(project_root: String) -> Result<CommandResult, String> {
    let root = resolve_project_root(&project_root)?;
    git(&root, &["push", "-u", "origin", "HEAD"])
}

#[tauri::command]
pub async fn git_fetch(project_root: String) -> Result<CommandResult, String> {
    let root = resolve_project_root(&project_root)?;
    git(&root, &["fetch"])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_relative_project_paths() {
        assert!(resolve_project_root("relative/path").unwrap_err().contains("absolute"));
    }

    #[test]
    fn reports_non_repository_folder() {
        let temp = tempfile::tempdir().unwrap();
        let result = tauri::async_runtime::block_on(get_git_workspace_status(
            temp.path().to_string_lossy().into_owned(),
        ))
        .unwrap();
        assert!(!result.is_repository);
        assert!(result.changes.is_empty());
    }
}
