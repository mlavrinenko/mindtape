//! The gitless `rm` strategies — what happens to inbound references when no
//! git repository pins a SWHID tombstone: `reject` (refuse, listing every
//! surviving referrer), `unlink` (strip the inbound edges, then delete), and
//! `retire` (keep the file, flip it to a terminal status). The git-backed
//! `git-tombstone` path lives in the parent module.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::Range;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use mindtape::{Config, RmStrategy, Task, link_edges, normalize_link, today_ymd};
use typst_cleanup::{Deletion, Plan, Rewrite, apply};

use crate::flip::status_edit;
use crate::task_edit::task_with;

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

/// The `reject` strategy's error text: every surviving task referencing a
/// removed task, each with the removed ids it links to.
#[must_use]
pub fn reject_message(referrers: &BTreeMap<PathBuf, HashSet<String>>) -> String {
    let references: usize = referrers.values().map(HashSet::len).sum();
    let mut lines = vec![format!(
        "cannot remove task(s): {references} reference(s) from surviving task(s) \
         (a gitless repository has no SWHID tombstone to rewrite onto; set \
         [rm] strategy to \"retire\" or \"unlink\" to remove them anyway)"
    )];
    for (file, ids) in referrers {
        let mut sorted: Vec<&str> = ids.iter().map(String::as_str).collect();
        sorted.sort();
        lines.push(format!(
            "  {}  links to {}",
            file.display(),
            sorted.join(", ")
        ));
    }
    lines.join("\n")
}

/// The gix no-repository message carries the caller's absolute task-root path;
/// swap it for a generic placeholder so the user's filesystem layout never
/// leaks into an error.
#[must_use]
pub fn redact_git_error(err: &typst_cleanup::Error, root: &Path) -> String {
    err.to_string()
        .replace(&root.display().to_string(), "<tasks-root>")
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

/// The `unlink` strategy's plan: delete the eligible task files, and strip
/// every inbound edge that references one from the surviving files. No git,
/// no tombstones.
///
/// # Errors
/// A survivor file that cannot be read, or a stripped source that cannot be
/// spliced back together, fails the run.
pub fn unlink_plan(tasks: &[Task], eligible: &HashSet<String>, root: &Path) -> Result<Plan> {
    let mut rewrites: Vec<Rewrite> = Vec::new();
    for task in tasks {
        if eligible.contains(&task.id) {
            continue;
        }
        let abs = root.join(&task.file);
        let source = std::fs::read_to_string(&abs)
            .with_context(|| format!("read {}", task.file.display()))?;
        let (ranges, links) = unlink_edits(&source, eligible);
        if ranges.is_empty() {
            continue;
        }
        let edits: Vec<typst_edit::Edit> = ranges
            .into_iter()
            .map(|range| typst_edit::Edit::new(range, String::new()))
            .collect();
        let contents =
            typst_edit::apply(&source, edits).map_err(|err| anyhow::anyhow!(err.to_string()))?;
        rewrites.push(Rewrite {
            abs,
            rel: task.file.clone(),
            contents,
            links,
        });
    }
    rewrites.sort_by(|a, b| a.rel.cmp(&b.rel));
    Ok(Plan {
        deletions: plain_delete_plan(tasks, eligible, root).deletions,
        rewrites,
    })
}

/// Which byte ranges of `source` to strip to unlink every removed task: whole
/// edge-helper calls (`depends-on("gone.typ") + related(...)[note]`) and
/// whole `link("gone.typ")` calls, each extended across a `+` operator (and
/// its whitespace) so no dangling operator survives. Returns the merged ranges
/// and how many edges were stripped.
fn unlink_edits(source: &str, eligible: &HashSet<String>) -> (Vec<Range<usize>>, usize) {
    let mut ranges: Vec<Range<usize>> = Vec::new();
    for call in mindtape::link_edges(source) {
        if call
            .targets
            .iter()
            .any(|target| eligible.contains(&normalize_link(&target.raw)))
        {
            ranges.push(call.range.clone());
        }
    }
    for call in typst_edit::find_calls(source, "link") {
        let targeted = call
            .args
            .iter()
            .filter(|arg| arg.name.is_none())
            .any(|arg| {
                source.as_bytes().get(arg.value_range.start) == Some(&b'"')
                    && eligible.contains(&normalize_link(&arg.value))
            });
        if targeted {
            ranges.push(call.range.clone());
        }
    }
    let stripped = ranges.len();
    if ranges.is_empty() {
        return (Vec::new(), 0);
    }
    let mut removals: Vec<Range<usize>> = ranges
        .into_iter()
        .map(|range| sweep_operator(range, source))
        .collect();
    removals.sort_by_key(|range| range.start);
    let mut merged: Vec<Range<usize>> = Vec::new();
    for removal in removals {
        match merged.last_mut() {
            Some(prev) if removal.start <= prev.end => prev.end = prev.end.max(removal.end),
            _ => merged.push(removal),
        }
    }
    // Stripped edges cover the whole `links:` value: drop the named argument.
    if let Some(range) = whole_links_edit(source, &merged) {
        return (std::iter::once(range).collect(), stripped);
    }
    (merged, stripped)
}

/// When the stripped edges cover the whole `links:` value, the range of the
/// named argument itself (separator comma and all) — dropping the edges alone
/// would leave a dangling `links: ` key.
fn whole_links_edit(source: &str, merged: &[Range<usize>]) -> Option<Range<usize>> {
    if let Ok(call) = task_with(source)
        && let Some(value) = call.named("links")
    {
        let value = value.value_range.clone();
        let bytes = source.as_bytes();
        let whole = value.clone().all(|byte| {
            bytes.get(byte).is_none_or(u8::is_ascii_whitespace)
                || merged.iter().any(|range| range.contains(&byte))
        });
        if whole {
            let before = bytes
                .iter()
                .take(value.start)
                .rposition(|byte| *byte == b',' || *byte == b'(')
                .map_or(value.start, |pos| pos + 1);
            let mut end = merged
                .iter()
                .filter(|range| range.start < value.end)
                .map(|range| range.end)
                .max()
                .unwrap_or(value.end);
            let mut cursor = end;
            while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
                cursor += 1;
            }
            if bytes.get(cursor) == Some(&b',') {
                end = cursor + 1;
            }
            return Some(before..end);
        }
    }
    None
}

/// Extend `range` across a `+` operator — `edge + range` or `range + edge` —
/// with its surrounding whitespace, so stripping the edge leaves no dangling
/// `+`. Both directions: a removed operand may sit anywhere in the sum.
fn sweep_operator(range: Range<usize>, source: &str) -> Range<usize> {
    let bytes = source.as_bytes();
    let mut start = range.start;
    let mut cursor = start;
    while cursor > 0 && bytes.get(cursor - 1).is_some_and(u8::is_ascii_whitespace) {
        cursor -= 1;
    }
    if cursor > 0 && bytes.get(cursor - 1) == Some(&b'+') {
        let mut back = cursor - 1;
        while back > 0 && bytes.get(back - 1).is_some_and(u8::is_ascii_whitespace) {
            back -= 1;
        }
        start = back;
    }
    let mut end = range.end;
    let mut ahead = end;
    while bytes.get(ahead).is_some_and(u8::is_ascii_whitespace) {
        ahead += 1;
    }
    if bytes.get(ahead) == Some(&b'+') {
        let mut forward = ahead + 1;
        while bytes.get(forward).is_some_and(u8::is_ascii_whitespace) {
            forward += 1;
        }
        end = forward;
    }
    Range { start, end }
}

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
            std::fs::read_to_string(&abs)
                .with_context(|| format!("read {}", task.file.display()))?
        } else {
            let source = std::fs::read_to_string(&abs)
                .with_context(|| format!("read {}", task.file.display()))?;
            status_edit(&source, &status, ymd, None)
                .with_context(|| format!("rewrite status in {}", abs.display()))?
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
        .context("the status legend declares no terminal status — set [retire] status")
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
                let rel = relative_target(from, dest);
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
        let contents = typst_edit::apply(&source, edits)
            .map_err(|err| anyhow::anyhow!(err.to_string()))
            .with_context(|| format!("retarget links in {}", task.file.display()))?;
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

/// The path of `to` (root-relative) seen from `from_dir` (root-relative), for
/// rewriting a link target when `retire` moves its file: `tasks/live.typ`
/// pointing at `tasks/.archive/gone.typ` becomes `.archive/gone.typ`.
fn relative_target(from_dir: &Path, to: &Path) -> PathBuf {
    let from: Vec<_> = from_dir.components().collect();
    let to: Vec<_> = to.components().collect();
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    let mut out = PathBuf::new();
    for _ in 0..(from.len() - common) {
        out.push("..");
    }
    for component in to.iter().skip(common) {
        out.push(component.as_os_str());
    }
    out
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
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
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
#[path = "rm_gitless_report.rs"]
mod report;
pub use report::{confirm_answer, confirm_retire, report_retire_applied, report_retire_plan};
