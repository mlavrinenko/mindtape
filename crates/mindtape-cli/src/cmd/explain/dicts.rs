//! `mt explain dicts`: list the project's dictionaries (`[dicts]` and the
//! built-ins), or the entries of one, as `in_dict()` and `@<dict>` read them.

use std::collections::BTreeMap;

use mindtape::{Dict, Dicts, Unknown};
use serde::Serialize;

use crate::cmd::schema::{DICT, DICTS};
use crate::diag::{Diag, Result};
use crate::render::envelope::Envelope;
use crate::util;

/// Arguments for `mt explain dicts`.
#[derive(Debug, clap::Args)]
pub struct DictsArgs {
    /// List this dictionary's entries instead of the dictionaries.
    pub name: Option<String>,
}

/// Run `mt explain dicts`.
///
/// # Errors
/// Returns an error if the project cannot be opened or the named dictionary is
/// unknown.
pub fn run(args: &DictsArgs, as_json: bool) -> Result<()> {
    let project = util::open()?;
    let dicts = project.config.compiled_dicts();
    let Some(name) = &args.name else {
        if as_json {
            let payload = list_json(dicts);
            println!(
                "{}",
                serde_json::to_string_pretty(&Envelope::new(DICTS, payload))?
            );
        } else {
            print!("{}", list(dicts));
        }
        return Ok(());
    };
    let dict = dicts.get(name).ok_or_else(|| unknown_dict(name, dicts))?;
    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&Envelope::new(DICT, detail_json(name, dict)))?
        );
    } else {
        dict.entries().for_each(|entry| println!("{entry}"));
    }
    Ok(())
}

/// One line per dictionary: its name, padded, then its count and case.
fn list(dicts: &Dicts) -> String {
    let width = dicts.names().map(str::len).max().unwrap_or(0);
    dicts
        .iter()
        .map(|(name, dict)| {
            format!(
                "{name:<width$}  {}, case {}\n",
                mindtape::count(dict.len(), "entry"),
                case(dict)
            )
        })
        .collect()
}

/// The object `mt explain dicts --json` prints: every dictionary's case and
/// entry count.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct DictsPayload {
    /// Each dictionary's summary, by name.
    pub dicts: BTreeMap<String, DictSummaryJson>,
}

/// One dictionary's case and entry count.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct DictSummaryJson {
    /// `fold` or `exact`.
    pub case: String,
    /// How many entries it resolved to.
    pub count: usize,
}

/// The object `mt explain dicts <name> --json` prints: one dictionary.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct DictDetailJson {
    /// The dictionary's name.
    pub name: String,
    /// `fold` or `exact`.
    pub case: String,
    /// How many entries it resolved to.
    pub count: usize,
    /// Its entries, as stored and sorted.
    pub entries: Vec<String>,
}

/// Each dictionary's name mapped to its case and entry count.
fn list_json(dicts: &Dicts) -> DictsPayload {
    DictsPayload {
        dicts: dicts
            .iter()
            .map(|(name, dict)| {
                let summary = DictSummaryJson {
                    case: case(dict).to_owned(),
                    count: dict.len(),
                };
                (name.to_owned(), summary)
            })
            .collect(),
    }
}

/// One dictionary with its entries, as `in_dict()` compares them.
fn detail_json(name: &str, dict: &Dict) -> DictDetailJson {
    DictDetailJson {
        name: name.to_owned(),
        case: case(dict).to_owned(),
        count: dict.len(),
        entries: dict.entries().map(ToOwned::to_owned).collect(),
    }
}

/// The `case` value as config spells it.
fn case(dict: &Dict) -> &'static str {
    match dict.case() {
        mindtape::DictCase::Fold => "fold",
        mindtape::DictCase::Exact => "exact",
    }
}

/// `unknown dict: '<name>'`, with the nearest name and every known one.
fn unknown_dict(name: &str, dicts: &Dicts) -> Diag {
    Diag::usage("unknown dict")
        .value(name)
        .unknown(Some(&Unknown::among(name, dicts.names())))
}
