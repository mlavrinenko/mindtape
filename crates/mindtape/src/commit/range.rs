//! A range of past commits, read the way `mt check --commit-msg` reads the
//! commit being written, so the same `[[commits.rules]]` hold them.
//!
//! Each commit is checked against its first parent only: the message is its
//! `%B`, the files are what it changes against that parent, and its
//! [`StatePlan`] puts the commit's tree as `staged` and the parent's as `head`,
//! with no working tree. A root commit's parent is the empty tree. A merge is
//! marked [`RangeCommit::is_merge`], so the CLI can skip it by default: it
//! joins commits that were checked on their own, and its first-parent diff
//! repeats them. `--check-merges` reads it like any other commit.
//!
//! Each state reads its whole tree ([`crate::commit::tasks`]), and a range
//! shares one [`crate::StateLoader`] across its commits, so an unchanged task
//! file is not evaluated again and a linear range reuses the previous commit's
//! `staged` state as the next commit's `head`. A [`StateSource::differs`] list
//! is no longer needed; the field stays empty for API compatibility.

use std::path::Path;

use crate::commit::tasks::{StatePlan, StateSource};
use crate::commit::{ChangedFiles, CommitMsgError, changed_paths, git, renamed_sources};

/// One commit of a range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeCommit {
    /// The full object id.
    pub id: String,
    /// The abbreviated id git would show.
    pub short: String,
    /// The first parent; `None` for a root commit.
    pub parent: Option<String>,
    /// The abbreviated first parent, for a report that names the parent
    /// state; `None` for a root commit.
    pub parent_short: Option<String>,
    /// Whether the commit has more than one parent.
    pub is_merge: bool,
}

/// A range's commits, oldest first, and what every commit's plan shares.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommitRange {
    /// Every commit `git rev-list <range>` lists, oldest first.
    pub commits: Vec<RangeCommit>,
    root_prefix: String,
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
        let mut commits: Vec<RangeCommit> = listing
            .lines()
            .filter_map(|line| {
                let mut ids = line.split(' ');
                let id = ids.next()?.to_owned();
                let short = ids.next()?.to_owned();
                let parents: Vec<&str> = ids.filter(|id| !id.is_empty()).collect();
                Some(RangeCommit {
                    id,
                    short,
                    parent: parents.first().map(|id| (*id).to_owned()),
                    parent_short: None,
                    is_merge: parents.len() > 1,
                })
            })
            .collect();
        for commit in &mut commits {
            if let Some(parent) = &commit.parent {
                commit.parent_short = Some(
                    git(root, &["rev-parse", "--short", parent])?
                        .trim()
                        .to_owned(),
                );
            }
        }
        Ok(Self {
            commits,
            root_prefix: git(root, &["rev-parse", "--show-prefix"])?
                .trim()
                .to_owned(),
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
                    differs: Vec::new(),
                },
                staged: StateSource {
                    rev: commit.id.clone(),
                    differs: Vec::new(),
                },
                worktree: false,
            },
        })
    }
}
