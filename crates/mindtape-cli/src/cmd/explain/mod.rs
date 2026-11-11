//! `mt explain` — the project's policy listings behind one command.
//!
//! Bare prints an index of every listing: the rules grouped by section, the
//! derived fields, and the tags, views and dictionaries the project declares.
//! A listing prints one of them in full, unchanged from the command it
//! replaces.

use crate::diag::Result;

pub(crate) mod add;
pub(crate) mod dicts;
pub(crate) mod index;
pub(crate) mod rules;
pub(crate) mod tags;
pub(crate) mod views;

/// Arguments for `explain`.
#[derive(Debug, clap::Args)]
pub struct ExplainArgs {
    /// Print JSON.
    ///
    /// Global, so `mt explain rules --json` reads the same as
    /// `mt explain --json rules`.
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub listing: Option<Listing>,
}

/// One listing `mt explain <listing>` prints in full.
#[derive(Debug, clap::Subcommand)]
pub enum Listing {
    /// List the `[[on.add]]` hooks, in the order `mt add` runs them.
    Add,
    /// List the rules and derived fields, one line each.
    ///
    /// Each line is a rule's severity, its name as `mt config show --origin`
    /// prints it, and the message a breach prints. `--for` keeps the rules
    /// one step enforces.
    Rules(rules::RulesArgs),
    /// List the tags your project allows and the rules that read them.
    ///
    /// A tag no rule reads is an area label. `--for` keeps the rules one step
    /// enforces, as `mt explain rules --for` does.
    Tags(tags::TagsArgs),
    /// List your saved views, or show one.
    Views(views::ViewsArgs),
    /// List your dictionaries, or the entries of one.
    Dicts(dicts::DictsArgs),
}

/// Run `mt explain`.
///
/// # Errors
/// Propagates the selected listing's failure.
pub fn run(args: &ExplainArgs) -> Result<()> {
    match &args.listing {
        None => index::run(args.json),
        Some(Listing::Add) => add::run(args.json),
        Some(Listing::Rules(rules)) => rules::run(rules, args.json),
        Some(Listing::Tags(tags)) => tags::run(tags, args.json),
        Some(Listing::Views(views)) => views::run(views, args.json),
        Some(Listing::Dicts(dicts)) => dicts::run(dicts, args.json),
    }
}
