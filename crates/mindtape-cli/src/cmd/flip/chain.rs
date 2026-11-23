//! The flips in progress above this `mt`, so a hook can't flip its way into
//! an endless loop.
//!
//! A `[[on.flip]]` command may run `mt flip` itself, and that nested flip runs
//! its own task's hooks, so no gate drops out. Every command runs with
//! [`ENV_CHAIN`] listing the flips in progress, outermost first, each a task
//! and the status it is taking. A nested flip that repeats one of them would
//! start the same hooks again, so `mt flip` refuses it and the refusal gives the
//! chain. Within one project that bounds a chain by tasks times statuses.
//! [`MAX_DEPTH`] bounds what no pair check can see, such as a hook that adds a
//! fresh task and flips it.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::diag::{Diag, Hint, Note};

/// The variable every `run` command gets: one flip in progress per line,
/// outermost first, as `<status>\t<id>\t<path>`.
pub(crate) const ENV_CHAIN: &str = "MINDTAPE_FLIP_CHAIN";

/// How many flips may be in progress above a flip before `mt flip` refuses it.
pub(crate) const MAX_DEPTH: usize = 16;

/// One flip in progress.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Link {
    /// The status the flip stamps.
    status: String,
    /// The task's id, for the refusal.
    id: String,
    /// The task file's absolute path, which tells two projects' tasks apart.
    path: String,
}

impl Link {
    fn new(path: &Path, id: &str, status: &str) -> Self {
        Self {
            status: status.to_owned(),
            id: id.to_owned(),
            path: path.to_string_lossy().into_owned(),
        }
    }

    /// The link as the refusal shows it: `alpha → done`.
    fn label(&self) -> String {
        format!("{} → {}", self.id, self.status)
    }

    /// The link as [`ENV_CHAIN`] spells it: `<status>\t<id>\t<path>`.
    fn line(&self) -> String {
        format!("{}\t{}\t{}", self.status, self.id, self.path)
    }
}

/// The flips in progress above this `mt`, outermost first.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Chain(Vec<Link>);

impl Chain {
    /// No flip in progress.
    #[cfg(test)]
    pub(crate) const NONE: Self = Self(Vec::new());

    /// The chain this process inherited, empty outside a hook.
    pub(crate) fn inherited() -> Self {
        std::env::var_os(ENV_CHAIN)
            .map(|raw| Self::parse(&raw.to_string_lossy()))
            .unwrap_or_default()
    }

    /// Read [`ENV_CHAIN`]'s value. The parser skips a line that isn't three
    /// fields, which [`MAX_DEPTH`] still counts on.
    fn parse(raw: &str) -> Self {
        Self(
            raw.lines()
                .filter_map(|line| {
                    let mut fields = line.splitn(3, '\t');
                    Some(Link {
                        status: fields.next()?.to_owned(),
                        id: fields.next()?.to_owned(),
                        path: fields.next()?.to_owned(),
                    })
                })
                .collect(),
        )
    }

    /// Refuse to flip the task at `path` into `status` when that flip is
    /// already in progress, or when the chain is [`MAX_DEPTH`] deep.
    ///
    /// # Errors
    /// The refusal, with the chain in a note.
    pub(crate) fn admit(&self, path: &Path, id: &str, status: &str) -> Result<(), Diag> {
        let link = Link::new(path, id, status);
        let refusal = if self.0.contains(&link) {
            Diag::refused("flip already in progress")
                .value(link.label())
                .hint(Hint::new(
                    "a hook can't flip the task it runs for into the same status again",
                ))
        } else if self.0.len() >= MAX_DEPTH {
            Diag::refused("flips nested too deep")
                .value(link.label())
                .hint(Hint::new(
                    "each hook's mt flip runs that task's hooks too; shorten the chain",
                ))
        } else {
            return Ok(());
        };
        Err(refusal.note(Note::new("in progress").name(self.labels())))
    }

    /// The chain's links as the refusal lists them, outermost first.
    fn labels(&self) -> String {
        let labels: Vec<String> = self.0.iter().map(Link::label).collect();
        labels.join(", ")
    }

    /// The [`ENV_CHAIN`] value for a command that runs while the task at
    /// `path` is taking `status`.
    pub(crate) fn passed_down(&self, path: &Path, id: &str, status: &str) -> OsString {
        self.passed_down_all(std::iter::once((path, id)), status)
    }

    /// The [`ENV_CHAIN`] value for a command that runs once a batch of tasks
    /// has taken `status`. Every landed flip joins the chain, so `mt` refuses a
    /// nested flip of one of them into the same status.
    pub(crate) fn passed_down_batch(&self, landed: &[(String, PathBuf)], status: &str) -> OsString {
        self.passed_down_all(
            landed
                .iter()
                .map(|(id, path)| (path.as_path(), id.as_str())),
            status,
        )
    }

    /// [`ENV_CHAIN`]'s value with `links` added above this chain, outermost
    /// first.
    fn passed_down_all<'p>(
        &self,
        links: impl Iterator<Item = (&'p Path, &'p str)>,
        status: &str,
    ) -> OsString {
        let added: Vec<Link> = links
            .map(|(path, id)| Link::new(path, id, status))
            .collect();
        let mut rows: Vec<String> = self.0.iter().chain(added.iter()).map(Link::line).collect();
        rows.push(String::new());
        rows.join("\n").into()
    }
}

#[cfg(test)]
#[path = "chain_tests.rs"]
mod tests;
