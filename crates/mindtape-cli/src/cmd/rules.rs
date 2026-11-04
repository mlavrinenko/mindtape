//! `mt rules` — list the rules the merged config applies, one line each.
//!
//! Read-only. Each line is the rule's severity, its name as
//! `mt config show --origin` prints it, and the message a breach prints.
//! `--for` keeps the rules the command at one moment enforces.

use mindtape::{ActiveRule, DerivedField, RuleSection, active_derived, active_rules, load_config};

use crate::diag::Result;
use crate::render::widest;
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
    pub(crate) const fn enforces(self, section: RuleSection) -> bool {
        match section {
            RuleSection::TasksRules | RuleSection::BodyHeaders | RuleSection::BodyContent => {
                matches!(self, Self::Add | Self::Flip)
            }
            RuleSection::CommitsRules => matches!(self, Self::Commit),
            RuleSection::OnFlip => matches!(self, Self::Flip),
        }
    }

    /// Whether this moment lists the derived fields. A commit rule cannot
    /// read `derived.` in v1, so `--for commit` hides them.
    const fn shows_derived(self) -> bool {
        !matches!(self, Self::Commit)
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
    let derived: Vec<DerivedField> = if args.moment.is_none_or(Moment::shows_derived) {
        active_derived(&config)
    } else {
        Vec::new()
    };
    if args.json {
        let document = serde_json::json!({
            "schema_version": mindtape::SCHEMA_VERSION,
            "rules": rules,
            "derived": derived,
        });
        println!("{}", serde_json::to_string_pretty(&document)?);
    } else {
        print!("{}", render(&rules, &derived));
    }
    Ok(())
}

/// One line per rule, `<severity>  <name>  <message>`, the first two padded
/// to their widest, then a `derived fields` block. One line saying no rule
/// applies when the project declares neither.
fn render(rules: &[ActiveRule], derived: &[DerivedField]) -> String {
    if rules.is_empty() && derived.is_empty() {
        return "no rules apply\n".to_owned();
    }
    let mut out = String::new();
    if !rules.is_empty() {
        let width = widest(rules.iter().map(|rule| rule.name.as_str()));
        for rule in rules {
            out.push_str(&format!(
                "{:<5}  {:<width$}  {}\n",
                rule.severity.label(),
                rule.name,
                message(rule)
            ));
        }
    }
    if !derived.is_empty() {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str("derived fields\n");
        let width = widest(derived.iter().map(|field| field.returns.as_str()));
        for field in derived {
            out.push_str(&derived_line(field, width));
        }
    }
    out
}

/// One line of the derived block: its type, field, kind, origin and readers.
/// A Typst collector's kind carries its module and function.
fn derived_line(field: &DerivedField, width: usize) -> String {
    let readers = if field.readers.is_empty() {
        "no rule".to_owned()
    } else {
        field.readers.join(", ")
    };
    let kind = match &field.detail {
        Some(detail) => format!("{} {detail}", field.kind),
        None => field.kind.clone(),
    };
    format!(
        "  {:<width$}  {}  {kind} from {}  read by {readers}\n",
        field.returns, field.name, field.origin
    )
}

/// The message a breach prints, or who words it when each breach words its
/// own.
pub(crate) fn message(rule: &ActiveRule) -> String {
    match (&rule.message, &rule.check) {
        (Some(message), _) => message.clone(),
        (None, Some(check)) => format!("(messages from {} in {})", check.function, check.typst),
        (None, None) => "(no message set)".to_owned(),
    }
}

#[cfg(test)]
#[path = "rules_tests.rs"]
mod tests;
