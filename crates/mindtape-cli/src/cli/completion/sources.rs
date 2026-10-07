//! How each [`super::Source`] resolves into completion candidates against the
//! live project.
//!
//! [`super::Source::attach`] hangs one of these resolvers on every value-taking
//! argument clap declares; the resolution itself — project discovery, the
//! per-source mapping and the quiet failure every source shares — lives here.
//! Every source resolves the project itself and yields no candidates when that
//! fails: a tab press lands on a command line the user is still typing, so a
//! diagnostic there reads as a broken shell rather than a broken project.
//!
//! Refs: `truth/tasks/emit-shell-completions-from-the-binary-that-answers-them.typ`

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::Path;
use std::sync::OnceLock;

use clap::builder::StyledStr;
use clap_complete::engine::CompletionCandidate;

use crate::util::{self, Project};

/// The loaded project, resolved at most once per completion process.
///
/// One tab press can ask several sources at once (`mt link <task> <kind>`
/// reaches both stems and kinds), and the load is the whole cost of answering
/// any of them. `None` means discovery or the config failed — every source
/// then yields nothing, which is the quiet failure this module promises.
pub(super) fn project() -> Option<&'static Project> {
    static PROJECT: OnceLock<Option<Project>> = OnceLock::new();
    PROJECT.get_or_init(|| util::open().ok()).as_ref()
}

/// Map the loaded project to candidates, or yield none when it did not load.
fn from_project(
    build: impl FnOnce(&Project) -> Vec<CompletionCandidate>,
) -> Vec<CompletionCandidate> {
    project().map(build).unwrap_or_default()
}

/// Task stems, each helped by its title.
///
/// A task whose own file failed to evaluate is simply absent: the load records
/// it as an error rather than a task, and a completion has nothing to say
/// about a file that does not parse.
pub(super) fn stems() -> Vec<CompletionCandidate> {
    from_project(|project| {
        project
            .outcome
            .tasks
            .iter()
            .map(|task| CompletionCandidate::new(&task.id).help(help(&task.title)))
            .collect()
    })
}

/// Legend values, each helped by its glyph — the same column `mt ls` prints,
/// so the candidate list reads like the table it will act on.
pub(super) fn statuses() -> Vec<CompletionCandidate> {
    from_project(|project| {
        project
            .config
            .tasks
            .status
            .entries
            .iter()
            .map(|entry| {
                CompletionCandidate::new(&entry.value).help(entry.glyph.as_deref().and_then(help))
            })
            .collect()
    })
}

/// View names under `prefix` — empty for `mt views <name>`, `@` for every
/// argument that takes the reference sugar.
pub(super) fn view_names(prefix: &str) -> Vec<CompletionCandidate> {
    from_project(|project| {
        project
            .config
            .views
            .iter()
            .map(|(name, spec)| {
                CompletionCandidate::new(format!("{prefix}{name}"))
                    .help(spec.description.as_deref().and_then(help))
            })
            .collect()
    })
}

/// Template stems, exactly the set `mt add --list` prints.
pub(super) fn templates() -> Vec<CompletionCandidate> {
    from_project(|project| {
        crate::cmd::add::values::list_templates(&project.config, &project.root)
            .into_iter()
            .map(CompletionCandidate::new)
            .collect()
    })
}

/// Edge kinds this project accepts, exactly the set `mt link` validates against.
pub(super) fn link_kinds() -> Vec<CompletionCandidate> {
    from_project(|project| {
        mindtape::link::kind_names(&project.config.tasks.links)
            .into_iter()
            .map(CompletionCandidate::new)
            .collect()
    })
}

/// The presets `mt preset ls` prints for `user_dir`: every built-in, helped
/// by its description, then each user preset not already offered as one.
///
/// A user directory that cannot be read yields the built-ins alone — the
/// quiet failure every other source keeps; `mt preset ls` does the reporting.
pub(super) fn presets_in(user_dir: Option<&Path>) -> Vec<CompletionCandidate> {
    let builtins = crate::cmd::init::preset::list_builtins();
    let users = user_dir
        .and_then(|dir| crate::cmd::init::preset::list_user_presets(dir).ok())
        .unwrap_or_default();
    let mut offered: Vec<CompletionCandidate> = builtins
        .iter()
        .map(|(name, description)| CompletionCandidate::new(*name).help(help(description)))
        .collect();
    offered.extend(
        users
            .into_iter()
            .filter(|name| !builtins.iter().any(|(builtin, _)| builtin == name))
            .map(CompletionCandidate::new),
    );
    offered
}

/// This project's tag vocabulary: `[tasks.tags] allowed` when it bounds one,
/// otherwise every tag its tasks actually carry, folder-derived ones included.
///
/// The bounded case wins because it is the set a write would be VALIDATED
/// against — offering a tag in use but no longer allowed would complete
/// straight into a refusal.
fn tags() -> BTreeSet<String> {
    let Some(project) = project() else {
        return BTreeSet::new();
    };
    let allowed = &project.config.tasks.tags.allowed;
    if !allowed.is_empty() {
        return allowed.iter().cloned().collect();
    }
    project
        .outcome
        .tasks
        .iter()
        .flat_map(|task| task.tags.iter().chain(task.derived_tags.iter()))
        .cloned()
        .collect()
}

/// `mt set`'s `<field>=<value>`: the fields it writes until the token carries
/// an operator, then whatever the key's own source offers.
///
/// A value completer sees the whole token, so every candidate it returns is a
/// whole token too — clap filters an [`ArgValueCandidates`] list by prefix, but
/// leaves a completer's output alone.
///
/// [`ArgValueCandidates`]: clap_complete::engine::ArgValueCandidates
pub(super) fn assignment(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return Vec::new();
    };
    if let Some(candidates) = tag_values(current) {
        return candidates;
    }
    if current.contains(['=', '@']) {
        return Vec::new();
    }
    crate::cmd::set::writable_fields()
        .filter(|name| name.starts_with(current))
        .map(|name| CompletionCandidate::new(format!("{name}=")))
        .collect()
}

/// The tag half on its own, for the token grammars whose keys are template
/// slots rather than a fixed field list.
pub(super) fn tag_token(current: &OsStr) -> Vec<CompletionCandidate> {
    current.to_str().and_then(tag_values).unwrap_or_default()
}

/// Candidates for a `tags=` / `tags+=` token, or `None` when the token names
/// some other key.
fn tag_values(current: &str) -> Option<Vec<CompletionCandidate>> {
    let typed = current
        .strip_prefix("tags+=")
        .or_else(|| current.strip_prefix("tags="))?;
    let head = &current[..current.len() - typed.len()];
    Some(
        tags()
            .into_iter()
            .filter(|tag| tag.starts_with(typed))
            .map(|tag| CompletionCandidate::new(format!("{head}{tag}")))
            .collect(),
    )
}

/// A candidate's help line, dropped when the text is empty — an empty help
/// renders as a dangling separator in zsh and fish.
fn help(text: &str) -> Option<StyledStr> {
    (!text.trim().is_empty()).then(|| StyledStr::from(text.to_owned()))
}

#[cfg(test)]
#[path = "sources_tests.rs"]
mod tests;
