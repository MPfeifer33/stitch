use std::path::Path;
use std::process::Command;
use serde::Serialize;
use walkdir::WalkDir;

use crate::StitchError;

/// All context gathered about a project.
#[derive(Debug, Serialize)]
pub struct ProjectContext {
    pub project_name: String,
    pub project_type: String,
    pub git_state: Option<GitState>,
    pub structure: Vec<FileEntry>,
    pub key_files: Vec<KeyFile>,
    pub recent_commits: Vec<CommitSummary>,
    pub evidence_sources: EvidenceSources,
}

#[derive(Debug, Serialize)]
pub struct GitState {
    pub branch: String,
    pub head_sha: String,
    pub dirty_count: usize,
}

#[derive(Debug, Serialize)]
pub struct FileEntry {
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct KeyFile {
    pub path: String,
    pub content: String,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct CommitSummary {
    pub sha: String,
    pub subject: String,
    pub date: String,
}

#[derive(Debug, Serialize)]
pub struct EvidenceSources {
    pub project_md: bool,
    pub readme: bool,
    pub witness: bool,
    pub latch: bool,
    pub probe: bool,
    pub atlas: bool,
}

/// Gather all available context about a project.
pub fn gather_context(repo: &Path, depth: usize, include_contents: bool) -> Result<ProjectContext, StitchError> {
    let project_name = repo.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let project_type = detect_project_type(repo);
    let git_state = gather_git_state(repo);
    let structure = gather_structure(repo, depth);
    let key_files = gather_key_files(repo, include_contents);
    let recent_commits = gather_recent_commits(repo, 10);
    let evidence_sources = detect_evidence_sources(repo);

    Ok(ProjectContext {
        project_name,
        project_type,
        git_state,
        structure,
        key_files,
        recent_commits,
        evidence_sources,
    })
}

/// Generate a compact brief for cold-start orientation.
pub fn generate_brief(ctx: &ProjectContext) -> String {
    let mut brief = Vec::new();

    brief.push(format!("Project: {} ({})", ctx.project_name, ctx.project_type));

    if let Some(ref git) = ctx.git_state {
        brief.push(format!("Branch: {} @ {}{}", git.branch, git.head_sha,
            if git.dirty_count > 0 { format!(" ({} dirty)", git.dirty_count) } else { String::new() }));
    }

    let file_count = ctx.structure.iter().filter(|f| !f.is_dir).count();
    let dir_count = ctx.structure.iter().filter(|f| f.is_dir).count();
    brief.push(format!("Structure: {} files, {} dirs", file_count, dir_count));

    if !ctx.recent_commits.is_empty() {
        brief.push(format!("Recent: {} ({})", ctx.recent_commits[0].subject, ctx.recent_commits[0].sha));
    }

    // Key files found
    for kf in &ctx.key_files {
        brief.push(format!("Found: {} ({})", kf.path, kf.reason));
    }

    // Evidence sources
    let mut sources = Vec::new();
    if ctx.evidence_sources.project_md { sources.push("PROJECT.md"); }
    if ctx.evidence_sources.witness { sources.push("witness"); }
    if ctx.evidence_sources.latch { sources.push("latch"); }
    if ctx.evidence_sources.probe { sources.push("probe"); }
    if ctx.evidence_sources.atlas { sources.push("atlas"); }
    if !sources.is_empty() {
        brief.push(format!("Evidence: {}", sources.join(", ")));
    }

    brief.join("\n")
}

fn detect_project_type(repo: &Path) -> String {
    if repo.join("Cargo.toml").exists() { "rust".to_string() }
    else if repo.join("package.json").exists() { "node".to_string() }
    else if repo.join("pyproject.toml").exists() { "python".to_string() }
    else if repo.join("go.mod").exists() { "go".to_string() }
    else { "unknown".to_string() }
}

fn gather_git_state(repo: &Path) -> Option<GitState> {
    let branch = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(repo)
        .output().ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())?;

    let head_sha = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(repo)
        .output().ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    let dirty_count = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo)
        .output().ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
        .unwrap_or(0);

    Some(GitState { branch, head_sha, dirty_count })
}

fn gather_structure(repo: &Path, max_depth: usize) -> Vec<FileEntry> {
    WalkDir::new(repo)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_str().unwrap_or("");
            !matches!(name, ".git" | "target" | "node_modules" | "__pycache__"
                | ".agent-witness" | ".agent-atlas" | ".agent-probe" | "dist" | "build")
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.path() != repo)
        .map(|e| {
            let rel = e.path().strip_prefix(repo).unwrap_or(e.path());
            FileEntry {
                path: rel.to_string_lossy().to_string(),
                is_dir: e.file_type().is_dir(),
                size: if e.file_type().is_file() { e.metadata().ok().map(|m| m.len()) } else { None },
            }
        })
        .collect()
}

fn gather_key_files(repo: &Path, include_contents: bool) -> Vec<KeyFile> {
    let key_paths = vec![
        ("PROJECT.md", "Project documentation"),
        ("README.md", "Project readme"),
        ("Cargo.toml", "Rust manifest"),
        ("package.json", "Node manifest"),
        ("pyproject.toml", "Python manifest"),
        ("go.mod", "Go manifest"),
    ];

    let mut found = Vec::new();
    for (path, reason) in key_paths {
        let full = repo.join(path);
        if full.exists() {
            let content = if include_contents {
                std::fs::read_to_string(&full).unwrap_or_else(|_| "(unreadable)".into())
            } else {
                "(use --contents to include)".into()
            };
            found.push(KeyFile {
                path: path.to_string(),
                content,
                reason: reason.to_string(),
            });
        }
    }
    found
}

fn gather_recent_commits(repo: &Path, limit: usize) -> Vec<CommitSummary> {
    let output = Command::new("git")
        .args(["log", &format!("-{}", limit), "--format=%h|%s|%ai"])
        .current_dir(repo)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.splitn(3, '|').collect();
                    if parts.len() >= 3 {
                        Some(CommitSummary {
                            sha: parts[0].to_string(),
                            subject: parts[1].to_string(),
                            date: parts[2].to_string(),
                        })
                    } else {
                        None
                    }
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn init_repo(dir: &std::path::Path) {
        std::process::Command::new("git").args(["init"]).current_dir(dir).output().unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        std::fs::write(dir.join("PROJECT.md"), "# Test Project").unwrap();
        std::process::Command::new("git").args(["add", "."]).current_dir(dir).output().unwrap();
        std::process::Command::new("git").args(["commit", "-m", "init"]).current_dir(dir).output().unwrap();
    }

    #[test]
    fn gather_context_basic() {
        let tmp = TempDir::new().unwrap();
        init_repo(tmp.path());
        let ctx = gather_context(tmp.path(), 3, false).unwrap();
        assert_eq!(ctx.project_type, "rust");
        assert!(!ctx.key_files.is_empty());
        assert!(ctx.git_state.is_some());
    }

    #[test]
    fn gather_context_with_contents() {
        let tmp = TempDir::new().unwrap();
        init_repo(tmp.path());
        let ctx = gather_context(tmp.path(), 3, true).unwrap();
        let project_md = ctx.key_files.iter().find(|f| f.path == "PROJECT.md");
        assert!(project_md.is_some());
        assert!(project_md.unwrap().content.contains("Test Project"));
    }

    #[test]
    fn generate_brief_includes_project_info() {
        let tmp = TempDir::new().unwrap();
        init_repo(tmp.path());
        let ctx = gather_context(tmp.path(), 3, false).unwrap();
        let brief = generate_brief(&ctx);
        assert!(brief.contains("rust"));
        assert!(brief.contains("PROJECT.md"));
    }

    #[test]
    fn detect_project_type_unknown() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(detect_project_type(tmp.path()), "unknown");
    }

    #[test]
    fn evidence_sources_detection() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("PROJECT.md"), "# test").unwrap();
        std::fs::create_dir(tmp.path().join(".agent-witness")).unwrap();
        let sources = detect_evidence_sources(tmp.path());
        assert!(sources.project_md);
        assert!(sources.witness);
        assert!(!sources.latch);
    }

    #[test]
    fn gather_structure_respects_depth() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("a/b/c")).unwrap();
        std::fs::write(tmp.path().join("a/b/c/deep.txt"), "deep").unwrap();
        let shallow = gather_structure(tmp.path(), 1);
        let deep = gather_structure(tmp.path(), 10);
        assert!(deep.len() >= shallow.len());
    }

    #[test]
    fn non_git_repo_returns_no_git_state() {
        let tmp = TempDir::new().unwrap();
        let state = gather_git_state(tmp.path());
        assert!(state.is_none());
    }
}

fn detect_evidence_sources(repo: &Path) -> EvidenceSources {
    EvidenceSources {
        project_md: repo.join("PROJECT.md").exists(),
        readme: repo.join("README.md").exists(),
        witness: repo.join(".agent-witness").exists(),
        latch: repo.join(".latch.db").exists(),
        probe: repo.join(".agent-probe").exists(),
        atlas: repo.join(".agent-atlas").exists(),
    }
}
