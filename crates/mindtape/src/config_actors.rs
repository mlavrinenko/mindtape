//! `[[actors]]` and `[[roles]]` — the project's actor registry.
//!
//! An actor is anyone who works tasks: a person or a language model, with an
//! optional set of declared roles. The registry is identity-only — a role
//! carries no tooling, prompt, or instruction payload; any tooling a role
//! should enable lives in the reader's harness config, keyed by role name by
//! convention. The assignee field references a declared actor by name (when
//! `[tasks.assignee]` `allowed` is set), and filters resolve an assignee's
//! roles from here at read time (`role == "reviewer"`), never from the task
//! marker itself.

use std::collections::HashSet;

use serde::Deserialize;

use crate::config::Config;

/// `[[actors]]` — one named worker in the project.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Actor {
    /// Unique actor name; the value an `assignee:` field (or a
    /// `[tasks.assignee] allowed` entry) references.
    pub name: String,
    /// What the actor is.
    pub kind: ActorKind,
    /// Roles the actor holds, each naming a declared `[[roles]]` entry.
    #[serde(default)]
    pub roles: Vec<String>,
}

/// What an actor is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ActorKind {
    /// A person.
    Human,
    /// A language model.
    Llm,
}

/// `[[roles]]` — one role name an actor can hold. A role carries no settings;
/// a harness keys its own config by role name.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Role {
    /// Unique role name.
    pub name: String,
}

/// The roles a declared actor holds, resolved from the registry at read time.
/// `None` when `name` is not a declared actor (or the registry is empty).
#[must_use]
pub fn roles_of<'a>(name: &str, config: &'a Config) -> Option<&'a [String]> {
    config
        .actors
        .iter()
        .find(|actor| actor.name == name)
        .map(|actor| actor.roles.as_slice())
}

/// Cross-checks for the actors/roles registry, folded into
/// [`Config::validate`]: role names non-empty and unique, actor names
/// non-empty and unique, every actor role referencing a declared `[[roles]]`
/// entry, and every `[tasks.assignee] allowed` value naming a declared actor.
/// The `kind` closed set is enforced at parse time by the enum itself.
pub(crate) fn validate_actors(config: &Config) -> Vec<String> {
    let mut problems = Vec::new();
    let mut role_names: HashSet<&str> = HashSet::new();
    for role in &config.roles {
        if role.name.trim().is_empty() {
            problems.push("roles: a role name must not be empty".to_owned());
        } else if !role_names.insert(role.name.as_str()) {
            problems.push(format!("roles: duplicate role `{}`", role.name));
        }
    }
    let mut actor_names: HashSet<&str> = HashSet::new();
    for actor in &config.actors {
        if actor.name.trim().is_empty() {
            problems.push("actors: an actor name must not be empty".to_owned());
        } else if !actor_names.insert(actor.name.as_str()) {
            problems.push(format!("actors: duplicate actor `{}`", actor.name));
        }
        for role in &actor.roles {
            if !config.roles.iter().any(|declared| declared.name == *role) {
                problems.push(format!(
                    "actors.{}: role `{role}` is not a declared role",
                    actor.name
                ));
            }
        }
    }
    for allowed in &config.tasks.assignee.allowed {
        if !config.actors.iter().any(|actor| actor.name == *allowed) {
            problems.push(format!(
                "tasks.assignee.allowed: `{allowed}` is not a declared actor"
            ));
        }
    }
    problems
}

#[cfg(test)]
#[path = "config_actors_tests.rs"]
mod tests;
