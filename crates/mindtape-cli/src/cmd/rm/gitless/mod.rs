//! The gitless `rm` strategies — what happens to inbound references when no
//! git repository pins a SWHID tombstone: `reject` (refuse, listing every
//! surviving referrer), `unlink` (strip the inbound edges, then delete), and
//! `retire` (keep the file, flip it to a terminal status). The git-backed
//! `git-tombstone` path lives in the parent module.
//!
//! # Files
//!
//! - `report.rs` — the retire strategy's report and confirm helpers.
//! - `unlink.rs` — the walk over the other files that name a task.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::diag::Result;
use mindtape::{Config, RmStrategy, Task, link_edges, normalize_link, today_ymd};
use typst_cleanup::{Deletion, Plan, Rewrite, apply};

use crate::cmd::flip::status_edit;
use crate::diag::{Diag, Hint, Note};
use crate::git;

/// The strategy for a run, resolved before anything is planned: the configured
/// `[rm] strategy` wins; unset, a detected git repository defaults to
/// `git-tombstone` (today's behaviour) and a gitless one to `reject` (the
/// consumer must opt into `retire` or `unlink`).
#[must_use]
pub fn resolve_strategy(config: &Config, has_git: bool) -> RmStrategy {
    config.rm.strategy.unwrap_or(if has_git {
        RmStrategy::GitTombstone
    } else {
        RmStrategy::Reject
    })
}

/// The `reject` strategy's refusal: a note for every surviving task that
/// references a removed task, naming the removed ids it links to.
pub fn reject_diag(referrers: &BTreeMap<PathBuf, HashSet<String>>) -> Diag {
    let refusal = Diag::refused("cannot remove tasks that other tasks link to").hint(Hint::new(
        "a gitless repository has no tombstone to rewrite onto: set [rm] strategy to \"retire\" or \"unlink\" to remove them anyway",
    ));
    referrers.iter().fold(refusal, |diag, (file, ids)| {
        let mut sorted: Vec<&str> = ids.iter().map(String::as_str).collect();
        sorted.sort();
        diag.note(Note::new("link from").name(format!(
            "{} to {}",
            file.display(),
            sorted.join(", ")
        )))
    })
}

/// The gix no-repository message carries the caller's absolute task-root path;
/// swap it for a generic placeholder so the user's filesystem layout never
/// leaks into an error.
#[must_use]
pub fn redact_git_error(err: &typst_cleanup::Error, root: &Path) -> String {
    err.to_string()
        .replace(&root.display().to_string(), "<tasks-root>")
}

/// Print the post-apply commit hint, but only when `root` sits inside a git
/// work tree. Every strategy's success report ends here — the delete-style
/// path (plain, git-tombstone, unlink) and the retire path — so gitless runs
/// never point the user at a tool they deliberately do not have. Reuses
/// [`git::toplevel`], the same discovery [`resolve_strategy`]'s caller already
/// runs to pick a default strategy, instead of a second probe.
pub fn print_commit_hint(root: &Path) {
    if git::toplevel(root).is_ok() {
        println!("review the changes and commit when satisfied");
    }
}

/// The deletion-only plan for tasks nothing references: no tombstones, no
/// rewrites, no git. Deletions sorted like [`typst_cleanup::plan`]'s, for a
/// stable report.
pub fn plain_delete_plan(tasks: &[Task], eligible: &HashSet<String>, root: &Path) -> Plan {
    let mut deletions: Vec<Deletion> = tasks
        .iter()
        .filter(|task| eligible.contains(&task.id))
        .map(|task| Deletion {
            abs: root.join(&task.file),
            rel: task.file.clone(),
        })
        .collect();
    deletions.sort_by(|a, b| a.rel.cmp(&b.rel));
    Plan {
        deletions,
        rewrites: Vec::new(),
    }
}

/// The inbound-edge walk `unlink` and `mt mv` share, split out for the
/// file-size gate.
mod unlink;
pub use unlink::{Retarget, inbound_rewrites, retarget_raw, unlink_plan};

/// The `retire` strategy's plan: keep the eligible task files, flip each to a
/// terminal status (the configured `[retire] status`, else the legend's first
/// terminal — an error if the legend declares none), and optionally move the
/// file into a sibling folder (`[retire] rename`). No git, no tombstones. A
/// task already in a terminal status is a tombstone already: it is only moved,
/// never re-stamped.
///
/// # Errors
/// A legend with no terminal status and no configured `[retire] status`, an
/// unreadable file, or a status rewrite that fails.
pub fn retire_plan(
    tasks: &[Task],
    eligible: &HashSet<String>,
    root: &Path,
    config: &Config,
) -> Result<RetirePlan> {
    let status = retire_status(config)?;
    let terminal = config.tasks.status.terminal();
    let rename = config.retire.rename.as_deref().map(PathBuf::from);
    let ymd = today_ymd();
    let mut rewrites: Vec<Rewrite> = Vec::new();
    let mut deletions: Vec<Deletion> = Vec::new();
    for task in tasks {
        if !eligible.contains(&task.id) {
            continue;
        }
        let abs = root.join(&task.file);
        let already = task
            .status
            .as_ref()
            .is_some_and(|status| terminal.contains(&status.name));
        let contents = if already {
            std::fs::read_to_string(&abs).map_err(|err| Diag::cannot_read(&task.file, &err))?
        } else {
            let source =
                std::fs::read_to_string(&abs).map_err(|err| Diag::cannot_read(&task.file, &err))?;
            status_edit(&source, &status, ymd, None).map_err(|err| {
                Diag::failed("can't rewrite the status")
                    .value(task.file.display().to_string())
                    .cause(&err)
            })?
        };
        let dest = match &rename {
            Some(dir) => match abs.file_name() {
                // The archive folder is a sibling of the task file: `tasks/x.typ`
                // moves into `tasks/.archive/x.typ`, not `<root>/.archive/…`.
                Some(name) => {
                    let parent = abs.parent().unwrap_or(Path::new("."));
                    parent.join(dir).join(name)
                }
                None => abs.clone(),
            },
            None => abs.clone(),
        };
        if dest == abs && already {
            continue;
        }
        if dest != abs {
            deletions.push(Deletion {
                abs: abs.clone(),
                rel: task.file.clone(),
            });
        }
        rewrites.push(Rewrite {
            abs: dest.clone(),
            rel: dest_rel(&rename, &task.file),
            contents,
            links: 0,
        });
    }
    rewrites.sort_by(|a, b| a.rel.cmp(&b.rel));
    deletions.sort_by(|a, b| a.rel.cmp(&b.rel));

    let link_rewrites = if rename.is_some() {
        retarget_inbound_edges(tasks, eligible, root, &rename)?
    } else {
        Vec::new()
    };

    Ok(RetirePlan {
        status,
        rename,
        rewrites,
        link_rewrites,
        deletions,
    })
}

/// The configured `[retire] status`, else the legend's first terminal status.
///
/// # Errors
/// A legend with no terminal status.
fn retire_status(config: &Config) -> Result<String> {
    if let Some(status) = &config.retire.status {
        return Ok(status.clone());
    }
    config
        .tasks
        .status
        .terminal()
        .into_iter()
        .next()
        .ok_or_else(|| {
            Diag::failed("the status legend declares no terminal status")
                .hint(Hint::new("set [retire] status, or mark a status terminal"))
        })
}

/// A move breaks path-based link resolution (`tasks/gone.typ` no longer
/// exists), so every surviving task's inbound edge to a moved file is
/// retargeted onto its new relative path — the tombstone must leave no
/// dangling edge, the same invariant the other strategies keep.
fn retarget_inbound_edges(
    tasks: &[Task],
    eligible: &HashSet<String>,
    root: &Path,
    rename: &Option<PathBuf>,
) -> Result<Vec<Rewrite>> {
    let mut dests: HashMap<String, PathBuf> = HashMap::new();
    for task in tasks.iter().filter(|task| eligible.contains(&task.id)) {
        dests.insert(task.id.clone(), dest_rel(rename, &task.file));
    }
    let mut link_rewrites: Vec<Rewrite> = Vec::new();
    for task in tasks {
        if eligible.contains(&task.id) {
            continue;
        }
        let abs = root.join(&task.file);
        let Ok(source) = std::fs::read_to_string(&abs) else {
            continue;
        };
        let mut edits: Vec<typst_edit::Edit> = Vec::new();
        for call in link_edges(&source) {
            for target in &call.targets {
                let Some(dest) = dests.get(&normalize_link(&target.raw)) else {
                    continue;
                };
                let from = task.file.parent().unwrap_or(Path::new(""));
                let rel = mindtape::relative_target(from, dest);
                edits.push(typst_edit::Edit::new(
                    target.range.clone(),
                    format!("\"{}\"", rel.display()),
                ));
            }
        }
        let links = edits.len();
        if links == 0 {
            continue;
        }
        let contents = typst_edit::apply(&source, edits).map_err(|err| {
            Diag::failed("can't retarget links")
                .value(task.file.display().to_string())
                .cause(&err)
        })?;
        link_rewrites.push(Rewrite {
            abs,
            rel: task.file.clone(),
            contents,
            links,
        });
    }
    link_rewrites.sort_by(|a, b| a.rel.cmp(&b.rel));
    Ok(link_rewrites)
}

/// The relative path a retired task file lands at: under the `rename` folder
/// (a sibling of the file) when set, else unchanged.
fn dest_rel(rename: &Option<PathBuf>, file: &Path) -> PathBuf {
    match rename {
        Some(dir) => match file.file_name() {
            Some(name) => {
                let parent = file.parent().unwrap_or(Path::new(""));
                parent.join(dir).join(name)
            }
            None => file.to_path_buf(),
        },
        None => file.to_path_buf(),
    }
}

/// The `retire` plan: `status` is the terminal status applied, `rename` the
/// optional archive folder, `rewrites` the files written (status flips, and
/// moves when `rename` is set), `link_rewrites` the surviving tasks whose
/// inbound edges were retargeted onto moved files, `deletions` the originals
/// of moved files.
#[derive(Debug)]
pub struct RetirePlan {
    pub status: String,
    pub rename: Option<PathBuf>,
    pub rewrites: Vec<Rewrite>,
    pub link_rewrites: Vec<Rewrite>,
    pub deletions: Vec<Deletion>,
}

/// Apply a [`RetirePlan`]: create the archive folder, write every rewrite
/// (flipped, moved, or retargeted), then delete the moved originals.
///
/// # Errors
/// Propagates the folder-creation or write failures.
pub fn retire_apply(plan: &RetirePlan) -> Result<()> {
    for rewrite in plan.rewrites.iter().chain(&plan.link_rewrites) {
        if let Some(parent) = rewrite.abs.parent() {
            std::fs::create_dir_all(parent).map_err(|err| {
                Diag::failed("can't create folder")
                    .value(parent.display().to_string())
                    .io(&err)
            })?;
        }
    }
    let mut all = plan.rewrites.clone();
    all.extend(plan.link_rewrites.clone());
    apply(&Plan {
        deletions: plan.deletions.clone(),
        rewrites: all,
    })?;
    Ok(())
}

/// The retire strategy's report/confirm helpers, split out for the
/// file-size gate.
mod report;
pub use report::{report_retire_applied, report_retire_plan, retire_prompt_lines};
