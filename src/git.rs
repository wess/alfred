use anyhow::{anyhow, Context, Result};
use std::process::{Command, Stdio};

#[derive(Debug, Default)]
pub struct GitStatus {
    pub branch: String,
    pub ahead: usize,
    pub behind: usize,
    pub staged: Vec<String>,
    pub unstaged: Vec<String>,
    pub untracked: Vec<String>,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Default)]
pub struct ConflictInfo {
    pub file: String,
    pub base: String,
    pub ours: String,
    pub theirs: String,
}

fn run(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| format!("Failed to execute git {}", args.first().unwrap_or(&"")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("warning") {
            return Err(anyhow!(
                "git {} failed: {}",
                args.first().unwrap_or(&""),
                stderr.trim()
            ));
        }
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn passthrough(args: &[String]) -> Result<i32> {
    let status = Command::new("git")
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| "Failed to execute git")?;

    Ok(status.code().unwrap_or(1))
}

pub fn is_git_repo() -> bool {
    run(&["rev-parse", "--git-dir"]).is_ok()
}

pub fn status() -> Result<GitStatus> {
    let branch = run(&["branch", "--show-current"]).unwrap_or_default();

    let (ahead, behind) = run(&["rev-list", "--left-right", "--count", "@{u}...HEAD"])
        .ok()
        .and_then(|s| {
            let parts: Vec<&str> = s.split('\t').collect();
            if parts.len() == 2 {
                let b = parts[0].parse().unwrap_or(0);
                let a = parts[1].parse().unwrap_or(0);
                Some((a, b))
            } else {
                None
            }
        })
        .unwrap_or((0, 0));

    let porcelain = run(&["status", "--porcelain=v1"]).unwrap_or_default();

    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut untracked = Vec::new();
    let mut conflicts = Vec::new();

    for line in porcelain.lines() {
        if line.len() < 3 {
            continue;
        }

        let index = line.chars().next().unwrap_or(' ');
        let worktree = line.chars().nth(1).unwrap_or(' ');
        let file = line[3..].to_string();

        // Check for conflicts
        if index == 'U'
            || worktree == 'U'
            || (index == 'A' && worktree == 'A')
            || (index == 'D' && worktree == 'D')
        {
            conflicts.push(file);
        } else if index == '?' {
            untracked.push(file);
        } else {
            if index != ' ' && index != '?' {
                staged.push(file.clone());
            }
            if worktree != ' ' && worktree != '?' {
                unstaged.push(file);
            }
        }
    }

    Ok(GitStatus {
        branch,
        ahead,
        behind,
        staged,
        unstaged,
        untracked,
        conflicts,
    })
}

pub fn diff(staged: bool) -> Result<String> {
    if staged {
        run(&["diff", "--cached"])
    } else {
        run(&["diff"])
    }
}

pub fn log(count: usize) -> Result<String> {
    run(&["log", "--oneline", &format!("-{}", count)])
}

pub fn get_branches() -> Result<Vec<String>> {
    let output = run(&["branch", "--format=%(refname:short)"])?;
    Ok(output.lines().filter(|s| !s.is_empty()).map(String::from).collect())
}

pub fn get_remote_branches() -> Result<Vec<String>> {
    let output = run(&["branch", "-r", "--format=%(refname:short)"])?;
    Ok(output.lines().filter(|s| !s.is_empty()).map(String::from).collect())
}

pub fn get_conflict_info(file: &str) -> Result<ConflictInfo> {
    let base = run(&["show", &format!(":1:{}", file)]).unwrap_or_default();
    let ours = run(&["show", &format!(":2:{}", file)]).unwrap_or_default();
    let theirs = run(&["show", &format!(":3:{}", file)]).unwrap_or_default();

    Ok(ConflictInfo {
        file: file.to_string(),
        base,
        ours,
        theirs,
    })
}

pub fn get_rebase_commits(onto: &str) -> Result<Vec<String>> {
    let output = run(&["log", "--oneline", &format!("{}..HEAD", onto)])?;
    Ok(output.lines().filter(|s| !s.is_empty()).map(String::from).collect())
}

pub fn commit(message: &str) -> Result<()> {
    run(&["commit", "-m", message])?;
    Ok(())
}

pub fn add(files: &[String]) -> Result<()> {
    let mut args = vec!["add"];
    for f in files {
        args.push(f);
    }
    run(&args)?;
    Ok(())
}

pub fn checkout(branch: &str) -> Result<()> {
    run(&["checkout", branch])?;
    Ok(())
}

pub fn create_branch(name: &str) -> Result<()> {
    run(&["checkout", "-b", name])?;
    Ok(())
}

pub fn delete_branch(name: &str, force: bool) -> Result<()> {
    let flag = if force { "-D" } else { "-d" };
    run(&["branch", flag, name])?;
    Ok(())
}

pub fn rebase(onto: &str, interactive: bool) -> Result<()> {
    if interactive {
        // For interactive rebase, we need to use inherit for stdin/stdout
        let status = Command::new("git")
            .args(["rebase", "-i", onto])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .with_context(|| "Failed to execute git rebase")?;

        if !status.success() {
            return Err(anyhow!("Rebase failed or has conflicts"));
        }
    } else {
        run(&["rebase", onto])?;
    }
    Ok(())
}

pub fn abort_rebase() -> Result<()> {
    run(&["rebase", "--abort"])?;
    Ok(())
}

pub fn continue_rebase() -> Result<()> {
    run(&["rebase", "--continue"])?;
    Ok(())
}

pub fn stage_file(file: &str) -> Result<()> {
    run(&["add", file])?;
    Ok(())
}

pub fn get_merged_branches(into: &str) -> Result<Vec<String>> {
    let output = run(&["branch", "--merged", into, "--format=%(refname:short)"])?;
    Ok(output
        .lines()
        .filter(|s| !s.is_empty() && *s != into && *s != "master" && *s != "main")
        .map(String::from)
        .collect())
}
