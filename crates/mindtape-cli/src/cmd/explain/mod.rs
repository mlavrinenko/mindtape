//! `mt explain` prints the project's policy listings behind one command.
//!
//! Bare prints an index of every listing: the rules grouped by section, the
//! derived fields, and the tags, views, dictionaries and templates the project
//! declares. A listing prints one of them in full, unchanged from the command
//! it replaces.

use serde::Serialize;

use crate::cmd::schema::SchemaId;
use crate::diag::Result;
use crate::render::envelope::Envelope;

pub(crate) mod add;
pub(crate) mod dicts;
pub(crate) mod index;
pub(crate) mod rules;
pub(crate) mod tags;
pub(crate) mod templates;
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
    /// An area label is a tag that only groups tasks. `--for` keeps the rules
    /// one step enforces, as `mt explain rules --for` does.
    Tags(tags::TagsArgs),
    /// List your saved views, or show one.
    Views(views::ViewsArgs),
    /// List your dictionaries, or the entries of one.
    Dicts(dicts::DictsArgs),
    /// List the templates `mt add` accepts, one line each.
    ///
    /// Each line gives the template's name, the `kind` it sets, its slot
    /// count, and `default` on the one `mt add` picks.
    Templates,
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
        Some(Listing::Templates) => templates::run(args.json),
    }
}

/// Print one listing: `payload` inside its `$schema` envelope under `--json`,
/// else the text `render` builds.
///
/// # Errors
/// Returns an error when the payload fails to serialize.
pub(crate) fn emit<T: Serialize>(
    as_json: bool,
    id: SchemaId,
    payload: T,
    render: impl FnOnce() -> String,
) -> Result<()> {
    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&Envelope::new(id, payload))?
        );
    } else {
        print!("{}", render());
    }
    Ok(())
}
