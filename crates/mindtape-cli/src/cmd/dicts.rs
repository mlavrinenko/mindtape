//! `mt dicts`: list the project's dictionaries (`[dicts]` and the built-ins),
//! or the entries of one, as `in_dict()` and `@<dict>` read them.

use mindtape::{Dict, Dicts, Unknown};

use crate::diag::{Diag, Result};
use crate::util;

/// Arguments for `dicts`.
#[derive(Debug, clap::Args)]
pub struct DictsArgs {
    /// List this dictionary's entries instead of the dictionaries.
    pub name: Option<String>,

    /// Print JSON.
    #[arg(long)]
    pub json: bool,
}

/// Run the `dicts` command.
///
/// # Errors
/// Returns an error if the project cannot be opened or the named dictionary is
/// unknown.
pub fn run(args: &DictsArgs) -> Result<()> {
    let project = util::open()?;
    let dicts = project.config.compiled_dicts();
    let Some(name) = &args.name else {
        if args.json {
            println!("{}", serde_json::to_string_pretty(&list_json(dicts))?);
        } else {
            print!("{}", list(dicts));
        }
        return Ok(());
    };
    let dict = dicts.get(name).ok_or_else(|| unknown_dict(name, dicts))?;
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&detail_json(name, dict))?
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

/// Each dictionary's name mapped to its case and entry count.
fn list_json(dicts: &Dicts) -> serde_json::Value {
    dicts
        .iter()
        .map(|(name, dict)| {
            let summary = serde_json::json!({"case": case(dict), "count": dict.len()});
            (name.to_owned(), summary)
        })
        .collect::<serde_json::Map<_, _>>()
        .into()
}

/// One dictionary with its entries, as `in_dict()` compares them.
fn detail_json(name: &str, dict: &Dict) -> serde_json::Value {
    serde_json::json!({
        "name": name,
        "case": case(dict),
        "count": dict.len(),
        "entries": dict.entries().collect::<Vec<_>>(),
    })
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
