//! `mt explain add` — the `[[on.add]]` hooks the merged config applies, in the
//! order they run, with what they do and where each came from.
//!
//! A hook a nearer layer replaced or `disable` dropped is absent, because the
//! listing reads the same merged config `mt add` does.

use mindtape::{ActiveAddHook, active_add, load_config_layered};
use serde::Serialize;

use crate::cmd::add::values::action_summary;
use crate::cmd::schema::EXPLAIN_ADD;
use crate::diag::Result;
use crate::render::envelope::Envelope;
use crate::render::widest;
use crate::util;

/// The JSON shape of one listed hook.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AddHookJson {
    /// `on.add.<name>`, or `on.add[<index>]` when unnamed.
    pub label: String,
    /// The hook's name, or `null` when unnamed.
    pub name: Option<String>,
    /// Its `when`, or the empty string when it fires every add.
    pub when: String,
    /// Each action in its one-line form.
    pub actions: Vec<String>,
    /// The layer or pack that supplied the hook.
    pub origin: String,
}

/// The JSON shape of `mt explain add --json`.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AddPayload {
    /// Every hook in merged order.
    pub hooks: Vec<AddHookJson>,
}

/// Run `mt explain add`.
///
/// # Errors
/// Returns an error if no project is found or its config fails to load.
pub fn run(as_json: bool) -> Result<()> {
    let root = util::resolve_root()?;
    let (config, _, origins) = load_config_layered(&root)?;
    let hooks = active_add(&config, &origins);
    if as_json {
        let payload = AddPayload {
            hooks: hooks.iter().map(|hook| hook_json(hook)).collect(),
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&Envelope::new(EXPLAIN_ADD, payload))?
        );
    } else {
        print!("{}", render(&hooks));
    }
    Ok(())
}

fn hook_json(hook: &ActiveAddHook<'_>) -> AddHookJson {
    AddHookJson {
        label: hook.label.clone(),
        name: hook.hook.name.clone(),
        when: hook.hook.when.clone(),
        actions: hook.hook.actions.iter().map(action_summary).collect(),
        origin: hook.origin.to_string(),
    }
}

/// One row per hook, aligned: label, `when`, actions, origin.
fn render(hooks: &[ActiveAddHook<'_>]) -> String {
    if hooks.is_empty() {
        return "no add hooks apply\n".to_owned();
    }
    let label_width = widest(hooks.iter().map(|hook| hook.label.as_str()));
    let when_width = widest(hooks.iter().map(|hook| when(hook)));
    let mut out = String::new();
    for hook in hooks {
        let actions: Vec<String> = hook.hook.actions.iter().map(action_summary).collect();
        out.push_str(&format!(
            "{:<label_width$}  {:<when_width$}  {}  {}\n",
            hook.label,
            when(hook),
            actions.join(", "),
            hook.origin,
        ));
    }
    out
}

fn when<'a>(hook: &ActiveAddHook<'a>) -> &'a str {
    if hook.hook.when.is_empty() {
        "-"
    } else {
        hook.hook.when.as_str()
    }
}

#[cfg(test)]
#[path = "add_tests.rs"]
mod tests;
