//! `mt rules` — list the rules the merged config applies, one line each.
//!
//! Read-only. Each line is the rule's severity, its name as
//! `mt config show --origin` prints it, and the message a breach prints.
//! `--for` keeps the rules the command at one moment enforces.

use mindtape::{ActiveRule, RuleSection, active_rules, load_config};

use crate::diag::Result;
use crate::util;

/// Arguments for `rules`.
#[derive(Debug, clap::Args)]
pub struct RulesArgs {
    /// List only the rules mt enforces at this moment.
    #[arg(long = "for", value_name = "MOMENT", value_enum)]
    pub moment: Option<Moment>,

    /// Print JSON.
    #[arg(long)]
    pub json: bool,
}

/// A moment whose command enforces a set of rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Moment {
    /// `mt add`, which checks the new task before it writes it.
    Add,
    /// `mt check --commit-msg` in the hook and `mt check --commits` on a range.
    Commit,
    /// `mt flip`, which checks the task and runs its `[[on.flip]]` actions.
    Flip,
}

impl Moment {
    /// Whether the command at this moment enforces the rules of `section`.
    ///
    /// `mt add` and `mt flip` both check the task they write through
    /// `pre_write::check`, which runs `[[tasks.rules]]` and both body rule
    /// arrays. Only `mt flip` runs `[[on.flip]]` (`cmd/flip/hooks.rs`), and
    /// only `mt check --commit-msg` and `--commits` read `[[commits.rules]]`
    /// (`cmd/check/commit.rs`, `cmd/check/range.rs`).
    const fn enforces(self, section: RuleSection) -> bool {
        match section {
            RuleSection::TasksRules | RuleSection::BodyHeaders | RuleSection::BodyContent => {
                matches!(self, Self::Add | Self::Flip)
            }
            RuleSection::CommitsRules => matches!(self, Self::Commit),
            RuleSection::OnFlip => matches!(self, Self::Flip),
        }
    }
}

/// Run the `rules` command.
///
/// # Errors
/// Returns an error if no project is found or its config fails to load.
pub fn run(args: &RulesArgs) -> Result<()> {
    let config = load_config(&util::resolve_root()?)?;
    let rules: Vec<ActiveRule> = active_rules(&config)
        .into_iter()
        .filter(|rule| {
            args.moment
                .is_none_or(|moment| moment.enforces(rule.section))
        })
        .collect();
    if args.json {
        let document = serde_json::json!({
            "schema_version": mindtape::SCHEMA_VERSION,
            "rules": rules,
        });
        println!("{}", serde_json::to_string_pretty(&document)?);
    } else {
        print!("{}", render(&rules));
    }
    Ok(())
}

/// One line per rule, `<severity>  <name>  <message>`, the first two padded
/// to their widest, or one line saying no rule applies.
fn render(rules: &[ActiveRule]) -> String {
    if rules.is_empty() {
        return "no rules apply\n".to_owned();
    }
    let width = rules
        .iter()
        .map(|rule| rule.name.chars().count())
        .max()
        .unwrap_or(0);
    rules
        .iter()
        .map(|rule| {
            format!(
                "{:<5}  {:<width$}  {}\n",
                rule.severity.label(),
                rule.name,
                message(rule)
            )
        })
        .collect()
}

/// The message a breach prints, or who words it when each breach words its
/// own.
fn message(rule: &ActiveRule) -> String {
    match (&rule.message, &rule.check) {
        (Some(message), _) => message.clone(),
        (None, Some(check)) => format!("(messages from {} in {})", check.function, check.typst),
        (None, None) => "(no message set)".to_owned(),
    }
}

#[cfg(test)]
#[path = "rules_tests.rs"]
mod tests;
