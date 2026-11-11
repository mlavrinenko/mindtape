//! `mt schema` — the JSON Schema of each machine output, printed or recorded.
//!
//! Every schema comes from the same Rust types the output serializes, so the
//! committed copies cannot drift from the binary. A schema's version changes
//! only when a change could break an earlier consumer; a compatible addition
//! updates the current file. The command works outside a project: it reads
//! and writes only `schema/output/` under the current directory.

use std::path::Path;

mod registry;
#[cfg(test)]
pub use registry::TASK;
pub use registry::{
    ADD, AT, AT_PROJECT, AT_SET, BOARD, CHECK, CONFIG_SHOW, DESCRIBE, DICT, DICTS, EVAL, EXPLAIN,
    EXPLAIN_ADD, GRAPH, LS, PACKS, PRESET_LS, RULES, SCHEMAS, SET_CHECK, SET_LS, Schema, TAGS,
    VIEW, VIEWS,
};

use clap::Args;

use crate::diag::{Diag, Hint, Result};

/// The base a working instance names in `$schema` unless
/// `MINDTAPE_SCHEMA_BASE` overrides it.
pub const CANONICAL_BASE: &str =
    "https://raw.githubusercontent.com/mlavrinenko/mindtape/main/schema/output";

/// Where `--write` records, relative to the current directory.
const OUTPUT_DIR: &str = "schema/output";

/// One machine output's identity: its name and shape version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaId {
    /// The output's identity, such as `ls`.
    pub name: &'static str,
    /// The shape version. Bump it only when a change could break an earlier
    /// consumer; a compatible addition updates the current version's file.
    pub version: u32,
}

impl SchemaId {
    /// The URL a working instance writes as `$schema`, honouring
    /// `MINDTAPE_SCHEMA_BASE`.
    #[must_use]
    pub fn url(self) -> String {
        format!("{}/{}", instance_base(), self.url_path())
    }

    /// The URL the schema itself names as `$id`, independent of the instance.
    #[must_use]
    pub fn canonical_url(self) -> String {
        format!("{CANONICAL_BASE}/{}", self.url_path())
    }

    /// `<name>/<version>.schema.json`: the path below the URL base.
    #[must_use]
    pub fn url_path(self) -> String {
        format!("{}/{}.schema.json", self.name, self.version)
    }

    /// `schema/output/<name>/<version>.schema.json`: the path from a
    /// project root.
    #[must_use]
    pub fn relative_path(self) -> String {
        format!("{OUTPUT_DIR}/{}", self.url_path())
    }
}

/// The base this instance names in `$schema`: `MINDTAPE_SCHEMA_BASE` with a
/// trailing `/` trimmed, or [`CANONICAL_BASE`] when unset or empty.
fn instance_base() -> String {
    let Ok(value) = std::env::var("MINDTAPE_SCHEMA_BASE") else {
        return CANONICAL_BASE.to_owned();
    };
    let value = value.trim_end_matches('/');
    if value.is_empty() {
        CANONICAL_BASE.to_owned()
    } else {
        value.to_owned()
    }
}

/// Arguments for `mt schema`.
#[derive(Debug, Args)]
pub struct SchemaArgs {
    /// The output whose schema to print, as `mt schema` lists it.
    #[arg(conflicts_with_all = ["write", "check"])]
    pub name: Option<String>,

    /// Write every current schema under `schema/output`.
    #[arg(long, conflicts_with = "check")]
    pub write: bool,

    /// Fail when a recorded schema differs from the generated one.
    #[arg(long)]
    pub check: bool,
}

/// Run `mt schema`.
///
/// # Errors
/// Returns an error if the name is unknown, a schema cannot be written, or a
/// recorded schema differs from the generated one.
pub fn run(args: &SchemaArgs) -> Result<()> {
    if args.write {
        return write_all(Path::new("."));
    }
    if args.check {
        return check_all(Path::new("."));
    }
    if let Some(name) = &args.name {
        print!("{}", find(name)?.json());
        Ok(())
    } else {
        print_index();
        Ok(())
    }
}

/// One line per registered schema: `<name>  <version>  <url>`.
fn print_index() {
    for schema in SCHEMAS {
        let id = schema.id;
        println!("{}  {}  {}", id.name, id.version, id.url());
    }
}

/// The registered schema `name` denotes.
///
/// # Errors
/// Returns an error naming the closest registered schema when `name` matches
/// none.
fn find(name: &str) -> Result<&'static Schema> {
    let known: Vec<&str> = SCHEMAS.iter().map(|schema| schema.id.name).collect();
    SCHEMAS
        .iter()
        .find(|schema| schema.id.name == name)
        .ok_or_else(|| {
            Diag::usage("unknown schema").value(name).did_you_mean_or(
                name,
                &known,
                Hint::new("run mt schema for the names"),
            )
        })
}

/// Write every registered schema under `root`, printing each path.
///
/// # Errors
/// Returns an error if a directory or file cannot be written.
fn write_all(root: &Path) -> Result<()> {
    for schema in SCHEMAS {
        let path = root.join(schema.id.relative_path());
        write_one(&path, &schema.json())?;
    }
    Ok(())
}

/// Write one schema file, creating its directory, and print the path.
///
/// # Errors
/// Returns an error if the directory or file cannot be written.
fn write_one(path: &Path, body: &str) -> Result<()> {
    crate::util::write_creating_parent(path, body)?;
    println!("{}", mindtape::display_path(path));
    Ok(())
}

/// Compare every registered schema with its recorded copy under `root`,
/// naming every missing or different file.
///
/// # Errors
/// Returns an error naming each stale file, or a recorded file that cannot be
/// read.
fn check_all(root: &Path) -> Result<()> {
    let mut stale = Vec::new();
    for schema in SCHEMAS {
        let path = root.join(schema.id.relative_path());
        match std::fs::read_to_string(&path) {
            Ok(recorded) if recorded == schema.json() => {}
            Ok(_) => stale.push(format!("{} differs", mindtape::display_path(&path))),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                stale.push(format!("{} is missing", mindtape::display_path(&path)));
            }
            Err(err) => return Err(Diag::cannot_read(&path, &err)),
        }
    }
    if stale.is_empty() {
        return Ok(());
    }
    Err(Diag::failed("recorded schemas are out of date")
        .values(stale)
        .hint(Hint::new("run mt schema --write")))
}

#[cfg(test)]
#[path = "../schema_tests.rs"]
mod tests;
