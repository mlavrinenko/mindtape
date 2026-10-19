//! Open every project of a set at once, for the commands that read a set:
//! [`open_set`] loads them in parallel and warns per member, naming it.

use std::path::PathBuf;
use std::sync::OnceLock;

use mindtape::LoadOutcome;

use crate::diag::{Diag, Hint, Kind, Result, emit};
use crate::util::{Project, config_warnings, held_back_warning, hyperlinks_for, load_problems};

/// The set a leading `@name` named, held until a command opens it: `main`
/// stores it ([`init_held_set`]), [`open_set`] callers read it
/// ([`held_set`]), and `util::open` refuses it ([`refuse_held`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeldSet {
    /// The set's name, without the `@`.
    pub name: String,
    /// The command word after the name, if any.
    pub command: Option<String>,
    /// Each project's name and root, in set order.
    pub members: Vec<(String, PathBuf)>,
}

impl HeldSet {
    /// The refusal of a command that runs on one project.
    pub fn refusal(&self) -> Diag {
        let command = self
            .command
            .as_deref()
            .map_or_else(|| "this command".to_owned(), |word| format!("`{word}`"));
        let held = mindtape::count(self.members.len(), "project");
        let diag = Diag::verbatim(
            Kind::Refused,
            format!(
                "{command} runs on one project, and @{} holds {held}",
                self.name
            ),
        );
        match self.members.first() {
            Some((first, _)) => {
                let example = match &self.command {
                    Some(word) => format!("mt @{first} {word}"),
                    None => format!("mt @{first}"),
                };
                diag.hint(Hint::as_in("name one project", example))
            }
            None => diag,
        }
    }
}

static HELD_SET: OnceLock<Option<HeldSet>> = OnceLock::new();

/// Record the held set (or none) for this process. Call once from `main`;
/// later calls are ignored.
pub fn init_held_set(held: Option<HeldSet>) {
    let _ = HELD_SET.set(held);
}

/// The held set, or `None` when the command line named a project or no `@`.
#[must_use]
pub fn held_set() -> Option<&'static HeldSet> {
    HELD_SET.get().and_then(Option::as_ref)
}

/// The refusal for a command that opens one project while a set is held.
///
/// # Errors
/// Returns [`HeldSet::refusal`] when a set is held.
pub fn refuse_held() -> Result<()> {
    held_set().map_or(Ok(()), |held| Err(held.refusal()))
}

/// One member of a set [`open_set`] opened: the name it goes by, and its
/// project, or why its config would not load.
#[derive(Debug)]
pub struct Member {
    /// The name the set gives the project, as `mt @<name>` takes it.
    pub name: String,
    /// The loaded project, or the error its config raised, unprinted.
    pub project: Result<Project>,
}

/// Load every `(name, root)` of a set in parallel ([`mindtape::load_many`]),
/// in the order given. Each member's config warnings and held-back commands
/// print as warnings that name it; its load errors are left to the caller, as
/// [`open`] leaves them, and a config that fails is that member's error.
#[must_use]
pub fn open_set(members: &[(String, PathBuf)]) -> Vec<Member> {
    let roots: Vec<PathBuf> = members.iter().map(|(_, root)| root.clone()).collect();
    let loaded = mindtape::load_many(&roots);
    members
        .iter()
        .zip(loaded)
        .map(|((name, root), loaded)| {
            let project = loaded.map_err(Diag::from).map(|loaded| Project {
                root: root.clone(),
                hyperlinks: hyperlinks_for(&loaded.config),
                config: loaded.config,
                outcome: loaded.outcome,
            });
            if let Ok(project) = &project {
                for diag in member_warnings(name, project) {
                    emit(&diag);
                }
            }
            Member {
                name: name.clone(),
                project,
            }
        })
        .collect()
}

/// [`open_set`], then print each member's load problems as warnings naming
/// it: the start of a command that lists what did load.
#[must_use]
pub fn open_set_warned(members: &[(String, PathBuf)]) -> Vec<Member> {
    let set = open_set(members);
    for member in &set {
        if let Ok(project) = &member.project {
            for diag in member_load_problems(&member.name, &project.outcome, Kind::Warning) {
                emit(&diag);
            }
        }
    }
    set
}

/// The config warnings of a set's member `name`, then the warning for the
/// commands its load held back, each naming it.
fn member_warnings(name: &str, project: &Project) -> Vec<Diag> {
    let held = mindtape::allow::take_held(&project.root);
    config_warnings(&project.root, &project.config)
        .into_iter()
        .chain(held_back_warning(&held))
        .map(|diag| diag.in_member(name))
        .collect()
}

/// [`load_problems`] of a set's member `name`, each naming it.
pub(crate) fn member_load_problems(name: &str, outcome: &LoadOutcome, kind: Kind) -> Vec<Diag> {
    load_problems(outcome, kind)
        .into_iter()
        .map(|diag| diag.in_member(name))
        .collect()
}

#[cfg(test)]
#[path = "project_set_tests.rs"]
mod tests;
