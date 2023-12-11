use super::cmd;
use super::errors;
use super::model::GitTreeDetails;
use super::model::GitTreeType;
use super::path;

/// Return Ok(garden::model::GitTreeDetails) for the specified path on success
/// or Err(garden::errors::CommandError) when Git commands error out.
pub fn worktree_details(pathbuf: &std::path::Path) -> Result<GitTreeDetails, errors::CommandError> {
    let mut worktree_count = 0;
    let cmd = ["git", "worktree", "list", "--porcelain"];
    let path = path::abspath(pathbuf);
    let exec = cmd::exec_in_dir(&cmd, &path);
    let output = cmd::stdout_to_string(exec)?;

    let worktree_token = "worktree ";
    let branch_token = "branch refs/heads/";
    let bare_token = "bare";

    let mut parent_path = std::path::PathBuf::new();
    let mut branch = String::new();
    let mut is_current = false;
    let mut is_bare = false;

    for line in output.lines() {
        if let Some(worktree) = line.strip_prefix(worktree_token) {
            let worktree_path = std::path::PathBuf::from(worktree);
            let current_path = path::abspath(&worktree_path);
            is_current = current_path == path;
            // The first worktree is the "parent" worktree.
            if worktree_count == 0 {
                parent_path = current_path;
            }
            worktree_count += 1;
        } else if is_current && line.starts_with(branch_token) {
            branch = line[branch_token.len()..].to_string();
        } else if is_current && line == bare_token {
            // Is this a bare repository?
            is_bare = true;
        }
    }

    // 0 or 1 worktrees implies that this is a regular worktree.
    // 0 doesn't happen in practice.
    if worktree_count < 2 {
        return Ok(GitTreeDetails {
            branch,
            tree_type: match is_bare {
                true => GitTreeType::Bare,
                false => GitTreeType::Tree,
            },
        });
    }

    if path == parent_path {
        return Ok(GitTreeDetails {
            branch,
            tree_type: GitTreeType::Parent,
        });
    }

    Ok(GitTreeDetails {
        branch,
        tree_type: GitTreeType::Worktree(parent_path),
    })
}

/// Return the current branch names for the specified repository path.
pub fn branches(path: &std::path::Path) -> Vec<String> {
    let mut branches: Vec<String> = Vec::new();
    let cmd = [
        "git",
        "for-each-ref",
        "--format=%(refname:short)",
        "refs/heads",
    ];
    let exec = cmd::exec_in_dir(&cmd, &path);
    if let Ok(output) = cmd::stdout_to_string(exec) {
        branches.append(
            &mut output
                .lines()
                .filter(|x| !x.is_empty())
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
        );
    }

    branches
}

/// Return the current branch name for the specified repository path.
pub fn branch(path: &std::path::Path) -> Option<String> {
    let cmd = ["git", "symbolic-ref", "--quiet", "--short", "HEAD"];
    let exec = cmd::exec_in_dir(&cmd, &path);
    if let Ok(output) = cmd::stdout_to_string(exec) {
        if !output.is_empty() {
            return Some(output);
        }
    }
    // Detached head? Show an abbreviated commit ID. This respects `git config core.abbrev`.
    let cmd = ["git", "rev-parse", "--short", "HEAD"];
    let exec = cmd::exec_in_dir(&cmd, &path);
    if let Ok(output) = cmd::stdout_to_string(exec) {
        if !output.is_empty() {
            return Some(output);
        }
    }
    // Unknown branch is an empty string.
    None
}
