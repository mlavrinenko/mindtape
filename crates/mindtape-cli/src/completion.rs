//! Shell completions answered by the binary that owns the values.
//!
//! `mt install` writes a few lines of shell that call `mt` back
//! (`clap_complete`'s `CompleteEnv`), so the static half of a completion is
//! clap's own parser and the dynamic half is the live project config. Nothing
//! is generated into a repository, so nothing can drift from one: a committed
//! script could state `--format json|csv` correctly forever and still be wrong
//! about every stem, status, view and template — the half that matters.
//!
//! ## Where an argument's values come from
//!
//! [`SOURCES`] is the one place that answers it, and it answers for EVERY
//! value-taking argument `mt` declares. [`Source::None`] is an answer too,
//! carrying the reason no project value fits. Two readers share the table:
//! [`decorate`] hangs each entry's completer on the clap tree, and
//! [`crate::cli_dump`] reflects it back out so `just check-doc-facts` can
//! set-difference the arguments clap declares against the ones named here. A
//! new completable argument therefore fails a gate until it says where its
//! values come from.
//!
//! An argument whose values are a `ValueEnum` is deliberately absent: clap
//! already completes a `possible_values` list from the parser itself, and
//! restating one here would be the committed-script mistake in miniature.
//!
//! ## Failing quietly
//!
//! Every source resolves the project itself and yields no candidates when that
//! fails. A tab press lands on a command line the user is still typing, so a
//! diagnostic there reads as a broken shell rather than a broken project;
//! `mt check` keeps the reporting and this module keeps silent.
//!
//! ## Status shortcut verbs
//!
//! `mt done <task>` reaches an ordinary run as a clap `external_subcommand`;
//! a completion run sees each legend verb as a real subcommand instead, see
//! `completion_verbs.rs`.
//!
//! Refs: `truth/tasks/emit-shell-completions-from-the-binary-that-answers-them.typ`

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::Path;
use std::sync::OnceLock;

use clap::builder::{StyledStr, ValueHint};
use clap::{Arg, Command, CommandFactory};
use clap_complete::CompleteEnv;
use clap_complete::engine::{ArgValueCandidates, ArgValueCompleter, CompletionCandidate};

use crate::cli::Cli;
use crate::util::{self, Project};

/// Where one argument's values come from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// Task stems — the project's own identities, helped by each title.
    Stems,
    /// The configured status legend (`[[tasks.status]]`).
    Statuses,
    /// Named views (`[views]`), bare.
    Views,
    /// Named views as the `@name` sugar filter, sort and column arguments take.
    ViewRefs,
    /// Template stems under `[templates] dirs`, plus the built-in `task`.
    Templates,
    /// The preset registry `mt preset ls` prints: the built-ins, then the
    /// user presets. Install-level, so it needs no project.
    Presets,
    /// Edge kinds: the six built-ins plus `[[tasks.links.kinds]]`.
    LinkKinds,
    /// `mt set`'s `<field>=<value>`: the fields it writes, then this project's
    /// tags once the token names `tags`.
    Assignment,
    /// `mt add`'s `--set` / `--` tokens: this project's tags once the token
    /// names `tags`. The keys are the chosen template's slots, which no
    /// completer can know — `-t` may sit anywhere on the line, including after
    /// the token being completed.
    TagToken,
    /// A filesystem path, completed through clap's own [`ValueHint`].
    FilePath,
    /// A directory, ditto.
    DirPath,
    /// No project value fits, and why.
    None(&'static str),
}

/// Every value-taking argument `mt` declares, keyed by `(command path, arg
/// id)` — the same flattened path [`crate::cli_dump`] builds, empty for a
/// global.
///
/// Ordered as `mt --help` lists the commands, so a reader comparing the two
/// walks them together.
pub const SOURCES: &[(&str, &str, Source)] = &[
    ("", "worktree", Source::DirPath),
    (
        "add",
        "title",
        Source::None("the task's title, in the user's own words"),
    ),
    ("add", "template", Source::Templates),
    ("add", "values", Source::FilePath),
    ("add", "set", Source::TagToken),
    ("add", "tokens", Source::TagToken),
    ("add", "into", Source::DirPath),
    (
        "add",
        "to",
        Source::None("a subfolder under `--into`, which may itself be typed later on the line"),
    ),
    (
        "add",
        "name",
        Source::None("the new task's stem: its identity, which no existing value predicts"),
    ),
    ("add", "status", Source::Statuses),
    (
        "add",
        "describe",
        Source::None("`text` or `json`, a literal of this flag's own and not a project value"),
    ),
    ("import", "input", Source::FilePath),
    ("import", "into", Source::DirPath),
    (
        "import",
        "to",
        Source::None("a subfolder under `--into`; see `mt add --to`"),
    ),
    ("import", "status", Source::Statuses),
    ("import", "map", Source::FilePath),
    ("ls", "filter", Source::ViewRefs),
    ("ls", "limit", Source::None("a count")),
    ("ls", "sort", Source::ViewRefs),
    ("ls", "columns", Source::ViewRefs),
    ("find", "terms", Source::None("the search text itself")),
    ("find", "limit", Source::None("a count")),
    ("find", "sort", Source::ViewRefs),
    ("find", "columns", Source::ViewRefs),
    ("graph", "filter", Source::ViewRefs),
    ("views", "name", Source::Views),
    ("init", "preset", Source::Presets),
    ("init", "schema", Source::FilePath),
    ("eval", "file", Source::FilePath),
    ("go", "id", Source::Stems),
    ("rm", "targets", Source::Stems),
    ("mv", "task", Source::Stems),
    (
        "mv",
        "new_stem",
        Source::None("the task's new identity, which no existing value predicts"),
    ),
    ("set", "task", Source::Stems),
    ("set", "assignment", Source::Assignment),
    ("review", "id", Source::Stems),
    ("flip", "status", Source::Statuses),
    ("flip", "tasks", Source::Stems),
    (
        "flip",
        "note",
        Source::None("the status note, in the user's own words"),
    ),
    ("recur", "task", Source::Stems),
    ("recur", "advance", Source::None("a count")),
    ("link", "task", Source::Stems),
    ("link", "kind", Source::LinkKinds),
    ("link", "targets", Source::Stems),
    (
        "link",
        "note",
        Source::None("the edge note, in the user's own words"),
    ),
    ("link", "apply", Source::FilePath),
    (
        "preset save",
        "name",
        Source::None("the new preset's name, which no existing value predicts"),
    ),
];

/// The source declared for `(path, id)`, or `None` when the table does not
/// name the argument at all — the state `just check-doc-facts` fails on.
#[must_use]
pub fn lookup(path: &str, id: &str) -> Option<Source> {
    SOURCES
        .iter()
        .find(|(cmd, arg, _)| *cmd == path && *arg == id)
        .map(|(_, _, source)| *source)
}

impl Source {
    /// Hang this source's completer on `arg`.
    fn attach(self, arg: Arg) -> Arg {
        match self {
            Self::Stems => arg.add(ArgValueCandidates::new(stems)),
            Self::Statuses => arg.add(ArgValueCandidates::new(statuses)),
            Self::Views => arg.add(ArgValueCandidates::new(|| view_names(""))),
            Self::ViewRefs => arg.add(ArgValueCandidates::new(|| view_names("@"))),
            Self::Templates => arg.add(ArgValueCandidates::new(templates)),
            Self::LinkKinds => arg.add(ArgValueCandidates::new(link_kinds)),
            Self::Presets => arg.add(ArgValueCandidates::new(|| {
                presets_in(crate::preset::user_preset_dir().as_deref())
            })),
            Self::Assignment => arg.add(ArgValueCompleter::new(assignment)),
            Self::TagToken => arg.add(ArgValueCompleter::new(tag_token)),
            Self::FilePath => arg.value_hint(ValueHint::FilePath),
            Self::DirPath => arg.value_hint(ValueHint::DirPath),
            Self::None(_) => arg,
        }
    }

    /// The source's name as `mt _cli --json` reports it; `None` for
    /// [`Source::None`], which reports its reason instead.
    #[must_use]
    pub fn label(self) -> Option<&'static str> {
        Some(match self {
            Self::Stems => "stems",
            Self::Statuses => "statuses",
            Self::Views => "views",
            Self::ViewRefs => "view-refs",
            Self::Templates => "templates",
            Self::LinkKinds => "link-kinds",
            Self::Presets => "presets",
            Self::Assignment => "assignment",
            Self::TagToken => "tag-token",
            Self::FilePath => "file-path",
            Self::DirPath => "dir-path",
            Self::None(_) => return None,
        })
    }
}

/// Answer a completion request and exit, or return and let `main` carry on.
///
/// Called first in `main`: `CompleteEnv` owns stdout for the duration of a
/// request, so anything printed ahead of it would arrive as a candidate.
pub fn complete() {
    if !requested() {
        return;
    }
    // A completion never sees the `-w` flag on the half-typed line — no parse
    // has happened yet — but `$MINDTAPE_WORKTREE` is exported by the shell and
    // steers every other command, so it steers this one too. Claimed only on a
    // completion run: the override is a one-shot, and taking it here on an
    // ordinary run would shadow the flag `main` is about to resolve.
    util::init_worktree(None);
    CompleteEnv::with_factory(|| decorate(Cli::command()))
        .var(COMPLETE_VAR)
        .complete();
}

/// The variable `CompleteEnv` activates on. Read here as well, so [`complete`]
/// can tell a completion run from an ordinary one before claiming anything.
const COMPLETE_VAR: &str = "COMPLETE";

/// Whether this process was spawned to answer a completion.
fn requested() -> bool {
    std::env::var_os(COMPLETE_VAR).is_some_and(|name| !name.is_empty() && name != "0")
}

/// Walk the clap tree and attach every [`SOURCES`] entry, then graft the
/// legend's status shortcut verbs (`mt done`, `mt wip`, …) on as real
/// subcommands so their task arguments complete too.
#[must_use]
pub fn decorate(cmd: Command) -> Command {
    verbs::graft(decorate_at(cmd, ""), &verbs::legend_verbs())
}

/// [`decorate`]'s recursion: `path` is the flattened command path of `cmd`.
fn decorate_at(mut cmd: Command, path: &str) -> Command {
    let ids: Vec<clap::Id> = cmd
        .get_arguments()
        .map(|arg| arg.get_id().clone())
        .collect();
    for id in ids {
        if let Some(source) = lookup(path, id.as_str()) {
            cmd = cmd.mut_arg(id, |arg| source.attach(arg));
        }
    }
    let names: Vec<String> = cmd
        .get_subcommands()
        .map(|sub| sub.get_name().to_owned())
        .collect();
    for name in names {
        let child = if path.is_empty() {
            name.clone()
        } else {
            format!("{path} {name}")
        };
        cmd = cmd.mut_subcommand(&name, |sub| decorate_at(sub, &child));
    }
    cmd
}

/// The loaded project, resolved at most once per completion process.
///
/// One tab press can ask several sources at once (`mt link <task> <kind>`
/// reaches both stems and kinds), and the load is the whole cost of answering
/// any of them. `None` means discovery or the config failed — every source
/// then yields nothing, which is the quiet failure this module promises.
fn project() -> Option<&'static Project> {
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
fn stems() -> Vec<CompletionCandidate> {
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
fn statuses() -> Vec<CompletionCandidate> {
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
fn view_names(prefix: &str) -> Vec<CompletionCandidate> {
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
fn templates() -> Vec<CompletionCandidate> {
    from_project(|project| {
        crate::add_values::list_templates(&project.config, &project.root)
            .into_iter()
            .map(CompletionCandidate::new)
            .collect()
    })
}

/// Edge kinds this project accepts, exactly the set `mt link` validates against.
fn link_kinds() -> Vec<CompletionCandidate> {
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
fn presets_in(user_dir: Option<&Path>) -> Vec<CompletionCandidate> {
    let builtins = crate::preset::list_builtins();
    let users = user_dir
        .and_then(|dir| crate::preset::list_user_presets(dir).ok())
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
fn assignment(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return Vec::new();
    };
    if let Some(candidates) = tag_values(current) {
        return candidates;
    }
    if current.contains(['=', '@']) {
        return Vec::new();
    }
    crate::set::writable_fields()
        .filter(|name| name.starts_with(current))
        .map(|name| CompletionCandidate::new(format!("{name}=")))
        .collect()
}

/// The tag half on its own, for the token grammars whose keys are template
/// slots rather than a fixed field list.
fn tag_token(current: &OsStr) -> Vec<CompletionCandidate> {
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

#[path = "completion_verbs.rs"]
mod verbs;

#[cfg(test)]
#[path = "completion_tests.rs"]
mod tests;
