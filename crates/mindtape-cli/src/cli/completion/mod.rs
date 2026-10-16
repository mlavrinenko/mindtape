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
//! [`crate::cli::tree_json`] reflects it back out so `just check-doc-facts` can
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
//! `cli/completion/verbs.rs`.
//!
//! # Files
//!
//! - `sources.rs` — the value sources a completion can draw on, and their lookups.
//! - `verbs.rs` — the status shortcut verbs a completion run sees.
//! - `cmd/ls/interactive/tests.rs`, `sources_tests.rs` — the table's coverage tests.
//!
//! Refs: `truth/tasks/emit-shell-completions-from-the-binary-that-answers-them.typ`

use clap::builder::ValueHint;
use clap::{Arg, Command, CommandFactory};
use clap_complete::CompleteEnv;
use clap_complete::engine::{ArgValueCandidates, ArgValueCompleter};

use self::sources::dict_names;
use self::sources::{
    assignment, link_kinds, presets_in, statuses, stems, tag_token, templates, view_names,
};
use crate::cli::Cli;
use crate::util;

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
    /// Dictionary names: `[dicts]` and the built-ins.
    Dicts,
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
/// id)` — the same flattened path [`crate::cli::tree_json`] builds, empty for a
/// global.
///
/// Ordered as `mt --help` lists the commands, so a reader comparing the two
/// walks them together.
// jscpd:ignore-start
// A table of `(command, argument, source)` rows: rows of one shape are the data,
// not copied logic, and a builder would only restate each row.
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
    ("check", "commit_msg", Source::FilePath),
    (
        "check",
        "commits",
        Source::None("a git revision range, which no project value predicts"),
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
    (
        "ls",
        "skip",
        Source::None("a count, usually copied from a prior truncated listing"),
    ),
    ("ls", "sort", Source::ViewRefs),
    ("ls", "columns", Source::ViewRefs),
    ("find", "terms", Source::None("the search text itself")),
    ("find", "limit", Source::None("a count")),
    (
        "find",
        "skip",
        Source::None("a count, usually copied from a prior truncated listing"),
    ),
    ("find", "sort", Source::ViewRefs),
    ("find", "columns", Source::ViewRefs),
    ("graph", "filter", Source::ViewRefs),
    ("views", "name", Source::Views),
    ("dicts", "name", Source::Dicts),
    ("init", "preset", Source::Presets),
    ("init", "schema", Source::FilePath),
    ("eval", "task", Source::Stems),
    ("go", "task", Source::Stems),
    ("rm", "targets", Source::Stems),
    ("mv", "task", Source::Stems),
    (
        "mv",
        "new_stem",
        Source::None("the task's new identity, which no existing value predicts"),
    ),
    ("set", "task", Source::Stems),
    ("set", "assignment", Source::Assignment),
    ("review", "task", Source::Stems),
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
        "_cli",
        "check_argv",
        Source::None("a command line the docs show, which no project value predicts"),
    ),
    (
        "_cli",
        "statuses",
        Source::None("the status verbs of the projects the docs show"),
    ),
    (
        "preset save",
        "name",
        Source::None("the new preset's name, which no existing value predicts"),
    ),
];
// jscpd:ignore-end

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
            Self::Dicts => arg.add(ArgValueCandidates::new(dict_names)),
            Self::Templates => arg.add(ArgValueCandidates::new(templates)),
            Self::LinkKinds => arg.add(ArgValueCandidates::new(link_kinds)),
            Self::Presets => arg.add(ArgValueCandidates::new(|| {
                presets_in(crate::cmd::init::preset::user_preset_dir().as_deref())
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
            Self::Dicts => "dicts",
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

mod sources;

mod verbs;

#[cfg(test)]
mod tests;
