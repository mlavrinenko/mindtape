//! A range of past commits, read the way `mt check --commit-msg` reads the
//! commit being written, so the same `[[commits.rules]]` hold them.
//!
//! Each commit is checked against its first parent only, a merge included:
//! the message is its `%B`, the files are what it changes against that
//! parent, and its [`StatePlan`] puts the commit's tree as `staged` and the
//! parent's as `head`, with no working tree. A root commit's parent is the
//! empty tree.
//!
//! The states are still overlays of the one working-tree load
//! ([`crate::commit_tasks`]), so each lists every path at which its revision
//! may differ from the working tree: where the parent differs from `HEAD`,
//! where `HEAD` differs from the working tree, and — for the commit — what the
//! commit changes. Listing a path that turns out not to differ costs a
//! re-read, never a wrong answer.

use std::path::Path;

use crate::commit_msg::{ChangedFiles, CommitMsgError, changed_paths, git, renamed_sources};
use crate::commit_tasks::{StatePlan, StateSource};
use crate::git_state::GitStatus;

/// One commit of a range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeCommit {
    /// The full object id.
    pub id: String,
    /// The abbreviated id git would show.
    pub short: String,
    /// The first parent; `None` for a root commit.
    pub parent: Option<String>,
}

/// A range's commits, oldest first, and what every commit's plan shares.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommitRange {
    /// Every commit `git rev-list <range>` lists, oldest first.
    pub commits: Vec<RangeCommit>,
    root_prefix: String,
    /// Every path at which the working tree differs from `HEAD`.
    dirty: Vec<String>,
    /// The empty tree's id: a root commit's parent.
    empty_tree: String,
}

/// One past commit, read: what [`crate::CommitRecord::build`] and
/// [`crate::CommitStates::load`] take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PastCommit {
    /// The raw message.
    pub message: String,
    /// What the commit changes against its first parent.
    pub changed: ChangedFiles,
    /// The commit as `staged`, its first parent as `head`, no working tree.
    pub plan: StatePlan,
}

impl CommitRange {
    /// List the commits `range` names — anything `git rev-list` reads, most
    /// often `<from>..<to>` — in the repository holding `root`.
    ///
    /// # Errors
    /// [`CommitMsgError`] when git cannot read the range or the repository.
    pub fn read(root: &Path, range: &str) -> Result<Self, CommitMsgError> {
        let listing = git(
            root,
            &[
                "rev-list",
                "--reverse",
                "--no-commit-header",
                "--format=%H %h %P",
                range,
                "--",
            ],
        )?;
        let commits = listing
            .lines()
            .filter_map(|line| {
                let mut ids = line.split(' ');
                Some(RangeCommit {
                    id: ids.next()?.to_owned(),
                    short: ids.next()?.to_owned(),
                    parent: ids
                        .next()
                        .filter(|id| !id.is_empty())
                        .map(ToOwned::to_owned),
                })
            })
            .collect();
        let status = GitStatus::read_unpaired(root).map_err(|stderr| CommitMsgError::Git {
            command: "status".to_owned(),
            stderr,
        })?;
        Ok(Self {
            commits,
            root_prefix: git(root, &["rev-parse", "--show-prefix"])?
                .trim()
                .to_owned(),
            dirty: status
                .entries()
                .iter()
                .map(|entry| entry.path.clone())
                .collect(),
            empty_tree: git(root, &["hash-object", "-t", "tree", "--stdin"])?
                .trim()
                .to_owned(),
        })
    }

    /// Read `commit`'s message, the files it changes and its plan.
    ///
    /// # Errors
    /// [`CommitMsgError`] when git cannot read the commit.
    pub fn commit(&self, root: &Path, commit: &RangeCommit) -> Result<PastCommit, CommitMsgError> {
        let parent = commit.parent.as_deref().unwrap_or(&self.empty_tree);
        let message = git(
            root,
            &[
                "rev-list",
                "--max-count=1",
                "--no-commit-header",
                "--format=%B",
                &commit.id,
            ],
        )?;
        let paths = changed_paths(root, &[parent, &commit.id])?;
        let renamed_from = renamed_sources(root, &[parent, &commit.id])?;
        let mut head = changed_paths(root, &[parent, "HEAD"])?;
        head.extend(self.dirty.iter().cloned());
        let mut staged = head.clone();
        staged.extend(paths.iter().cloned());
        // A path listed twice would be overlaid, and so loaded, twice.
        for differs in [&mut head, &mut staged] {
            differs.sort_unstable();
            differs.dedup();
        }
        Ok(PastCommit {
            message,
            changed: ChangedFiles {
                paths,
                renamed_from,
                root_prefix: self.root_prefix.clone(),
            },
            plan: StatePlan {
                root_prefix: self.root_prefix.clone(),
                head: StateSource {
                    rev: parent.to_owned(),
                    differs: head,
                },
                staged: StateSource {
                    rev: commit.id.clone(),
                    differs: staged,
                },
                worktree: false,
            },
        })
    }
}
