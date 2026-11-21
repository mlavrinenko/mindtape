//! `mt dicts`: list the project's dictionaries (`[dicts]` and the built-ins),
//! or the entries of one, as `in_dict()` and `@<dict>` read them.

use std::collections::BTreeMap;

use mindtape::{Dict, DictSource, Dicts, Unknown};
use serde::Serialize;

use crate::cmd::schema::{DICT, DICTS};
use crate::diag::{Diag, Result};
use crate::render::envelope::Envelope;
use crate::util;

/// Arguments for `mt dicts`.
#[derive(Debug, clap::Args)]
pub struct DictsArgs {
    /// List this dictionary's entries instead of the dictionaries.
    pub name: Option<String>,

    /// Print JSON.
    #[arg(long)]
    pub json: bool,
}

/// Run `mt dicts`.
///
/// # Errors
/// Returns an error if the project fails to open or the named dictionary is
/// unknown.
pub fn run(args: &DictsArgs) -> Result<()> {
    let project = util::open()?;
    let dicts = project.config.compiled_dicts();
    let as_json = args.json;
    let Some(name) = args.name.as_deref() else {
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

/// Each line holds a dictionary's name, padded, then its count, case and
/// source.
fn list(dicts: &Dicts) -> String {
    let name_width = dicts.names().map(str::len).max().unwrap_or(0);
    let count_width = dicts
        .iter()
        .map(|(_, dict)| dict.len().to_string().len())
        .max()
        .unwrap_or(1);
    let case_width = dicts
        .iter()
        .map(|(_, dict)| case(dict).len())
        .max()
        .unwrap_or(0);
    dicts
        .iter()
        .map(|(name, dict)| {
            format!(
                "{name:<name_width$}  {count:<count_width$}  case {case:<case_width$}  {source}\n",
                count = dict.len(),
                case = case(dict),
                source = source(dict),
            )
        })
        .collect()
}

/// The source column: `inline`, or the names it includes.
fn source(dict: &Dict) -> String {
    match dict.source() {
        DictSource::Inline => "inline".to_owned(),
        DictSource::Include(names) => format!("include: {}", names.join(", ")),
    }
}

/// The object `mt dicts --json` prints: every dictionary's case, entry count
/// and source.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct DictsPayload {
    /// Each dictionary's summary, by name.
    pub dicts: BTreeMap<String, DictSummaryJson>,
}

/// One dictionary's case, entry count and source.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct DictSummaryJson {
    /// `fold` or `exact`.
    pub case: String,
    /// How many entries it resolved to.
    pub count: usize,
    /// `inline`, or `include` when it unions in other dictionaries.
    pub source: String,
    /// The dictionaries it unions in, empty for an `inline` one.
    pub include: Vec<String>,
}

/// The object `mt dicts <name> --json` prints: one dictionary.
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

/// Each dictionary's name mapped to its case, entry count and source.
fn list_json(dicts: &Dicts) -> DictsPayload {
    DictsPayload {
        dicts: dicts
            .iter()
            .map(|(name, dict)| {
                let summary = DictSummaryJson {
                    case: case(dict).to_owned(),
                    count: dict.len(),
                    source: source_word(dict).to_owned(),
                    include: includes(dict).to_owned(),
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

/// The `source` value: `inline` or `include`.
fn source_word(dict: &Dict) -> &'static str {
    match dict.source() {
        DictSource::Inline => "inline",
        DictSource::Include(_) => "include",
    }
}

/// The dictionaries this one includes, empty when the source is `inline`.
fn includes(dict: &Dict) -> &[String] {
    match dict.source() {
        DictSource::Inline => &[],
        DictSource::Include(names) => names,
    }
}

/// `unknown dict: '<name>'`, with the nearest name and every known one.
fn unknown_dict(name: &str, dicts: &Dicts) -> Diag {
    Diag::usage("unknown dict")
        .value(name)
        .unknown(Some(&Unknown::among(name, dicts.names())))
}
