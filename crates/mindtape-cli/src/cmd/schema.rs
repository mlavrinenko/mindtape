//! `mt schema` — the JSON Schema of each machine output, printed or recorded.
//!
//! Every schema comes from the same Rust types the output serializes, so the
//! committed copies cannot drift from the binary. A schema's version bumps
//! only when that output's shape changes. The command works outside a
//! project: it reads and writes only `schema/output/` under the current
//! directory.

use std::path::Path;

use clap::Args;
use serde_json::Value;

use crate::cmd::add::output::AddReport;
use crate::cmd::add::values::DescribePayload;
use crate::cmd::at::{AtListPayload, AtProjectJson, AtSetJson};
use crate::cmd::board::BoardPayload;
use crate::cmd::config::ConfigShowPayload;
use crate::cmd::explain::dicts::{DictDetailJson, DictsPayload};
use crate::cmd::explain::views::ViewsPayload;
use crate::cmd::packs::PacksPayload;
use crate::diag::{Diag, Hint, Result};
use crate::render::envelope::Envelope;
use crate::render::objects::{
    CheckPayload, CheckSetPayload, ExplainIndexPayload, GraphPayload, PresetsPayload, RulesPayload,
    TagsPayload,
};
use crate::render::output::SetLsPayload;
use crate::render::payload::LsPayload;
use mindtape::{CanonicalTask, ViewSpec};

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
    /// The shape version, bumped only when the output changes.
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

// jscpd:ignore-start
// A registry of ids and generated schemas: one entry per output, with no
// helper to extract. The comment in `.just/lint.just` covers these lists.

/// `mt ls --json`.
pub const LS: SchemaId = SchemaId {
    name: "ls",
    version: 1,
};

/// One task record, as `mt ls --json` prints it and `mt import` reads it.
pub const TASK: SchemaId = SchemaId {
    name: "task",
    version: 1,
};

/// `mt check --json`.
pub const CHECK: SchemaId = SchemaId {
    name: "check",
    version: 1,
};

/// `mt @name check --json`.
pub const SET_CHECK: SchemaId = SchemaId {
    name: "set-check",
    version: 1,
};

/// `mt graph --json`.
pub const GRAPH: SchemaId = SchemaId {
    name: "graph",
    version: 1,
};

/// `mt preset ls --json`.
pub const PRESET_LS: SchemaId = SchemaId {
    name: "preset-ls",
    version: 1,
};

/// `mt explain --json`.
pub const EXPLAIN: SchemaId = SchemaId {
    name: "explain",
    version: 1,
};

/// `mt explain rules --json`.
pub const RULES: SchemaId = SchemaId {
    name: "rules",
    version: 1,
};

/// `mt explain tags --json`.
pub const TAGS: SchemaId = SchemaId {
    name: "tags",
    version: 1,
};

/// `mt eval --json`.
pub const EVAL: SchemaId = SchemaId {
    name: "eval",
    version: 1,
};

/// `mt add --json`.
pub const ADD: SchemaId = SchemaId {
    name: "add",
    version: 1,
};

// jscpd:ignore-end

// jscpd:ignore-start
const LS_TITLE: &str = "Mindtape listing";
const LS_DESCRIPTION: &str = "The object `mt ls --json` prints: its schema URL, the task records, and how the listing was shaped, cut and matched against retired stems.";
const TASK_TITLE: &str = "Mindtape task";
const TASK_DESCRIPTION: &str =
    "One task record as `mt ls --json` prints it and `mt import` reads it.";
const CHECK_TITLE: &str = "Mindtape check";
const CHECK_DESCRIPTION: &str =
    "The object `mt check --json` prints: the problems it found, in print order.";
const SET_CHECK_TITLE: &str = "Mindtape set check";
const SET_CHECK_DESCRIPTION: &str =
    "The object `mt @name check --json` prints: one report per project in the set.";
const GRAPH_TITLE: &str = "Mindtape graph";
const GRAPH_DESCRIPTION: &str = "The object `mt graph --json` prints: its task nodes, the typed edges between them, and the links whose kind points outside the backlog.";
const PRESET_LS_TITLE: &str = "Mindtape presets";
const PRESET_LS_DESCRIPTION: &str = "The object `mt preset ls --json` prints: every built-in and user preset, with the shadowing between them.";
const EXPLAIN_TITLE: &str = "Mindtape explain index";
const EXPLAIN_DESCRIPTION: &str = "The object `mt explain --json` prints: the rules by section, the derived fields, and the tag, view and dictionary names.";
const RULES_TITLE: &str = "Mindtape rules";
const RULES_DESCRIPTION: &str =
    "The object `mt explain rules --json` prints: the active rules and derived fields.";
const TAGS_TITLE: &str = "Mindtape tags";
const TAGS_DESCRIPTION: &str =
    "The object `mt explain tags --json` prints: each allowed tag and the rules that read it.";
const EVAL_TITLE: &str = "Mindtape eval";
const EVAL_DESCRIPTION: &str =
    "The object `mt eval --json` prints: one task record inside its `$schema` envelope.";
const ADD_TITLE: &str = "Mindtape add report";
const ADD_DESCRIPTION: &str =
    "The object `mt add --json` prints: what the run did, the task's path and its stem hash.";
const BOARD_TITLE: &str = "Mindtape board";
const BOARD_DESCRIPTION: &str = "The object `mt board --json` prints: one row per project, its cells, or the error that kept it from loading.";
const AT_TITLE: &str = "Mindtape at entries";
const AT_DESCRIPTION: &str = "The object `mt at --json` prints: every `at.toml` entry, its include and exclude patterns and its forks policy.";
const AT_PROJECT_TITLE: &str = "Mindtape at project";
const AT_PROJECT_DESCRIPTION: &str = "The object `mt at <name> --json` prints for a project: its name, the `project` kind and its path.";
const AT_SET_TITLE: &str = "Mindtape at set";
const AT_SET_DESCRIPTION: &str =
    "The object `mt at <name> --json` prints for a set: its name, the `set` kind and its members.";
const SET_LS_TITLE: &str = "Mindtape set listing";
const SET_LS_DESCRIPTION: &str = "The object `mt @set ls --json` prints: one `mt ls --json` listing per project, with its name and error.";
const PACKS_TITLE: &str = "Mindtape pack install";
const PACKS_DESCRIPTION: &str = "The object `mt packs install --json` prints: each locked source, its commit and tree hash, and whether this run fetched it.";
const DESCRIBE_TITLE: &str = "Mindtape template slots";
const DESCRIBE_DESCRIPTION: &str = "The object `mt add --describe=json` prints: the template's slots, each with its kind, default and choices.";
const VIEWS_TITLE: &str = "Mindtape views";
const VIEWS_DESCRIPTION: &str =
    "The object `mt explain views --json` prints: every named view's spec, by name.";
const VIEW_TITLE: &str = "Mindtape view";
const VIEW_DESCRIPTION: &str = "The object `mt explain views <name> --json` prints: one view's filter, sort, columns and description.";
const DICTS_TITLE: &str = "Mindtape dictionaries";
const DICTS_DESCRIPTION: &str =
    "The object `mt explain dicts --json` prints: each dictionary's case and entry count, by name.";
const DICT_TITLE: &str = "Mindtape dictionary";
const DICT_DESCRIPTION: &str = "The object `mt explain dicts <name> --json` prints: one dictionary's name, case, count and entries.";
const CONFIG_SHOW_TITLE: &str = "Mindtape effective config";
const CONFIG_SHOW_DESCRIPTION: &str = "The object `mt config show --json` prints: the merged config under `config`, shaped as `.mindtape/config.toml`.";
// jscpd:ignore-end

/// One registered output schema.
#[derive(Debug)]
pub struct Schema {
    /// The output's name and version.
    pub id: SchemaId,
    /// The schema's title.
    pub title: &'static str,
    /// The schema's description.
    pub description: &'static str,
    /// The schema the output's Rust type generates.
    schema: fn() -> schemars::Schema,
}

impl Schema {
    /// The generated schema stamped with its `$id`, title and description as
    /// pretty JSON with a trailing newline.
    #[must_use]
    pub fn json(&self) -> String {
        stamped((self.schema)(), self.id, self.title, self.description)
    }
}

// jscpd:ignore-start
/// Every registered schema, in the order `mt schema` lists them.
pub const SCHEMAS: &[Schema] = &[
    Schema {
        id: LS,
        title: LS_TITLE,
        description: LS_DESCRIPTION,
        schema: || schemars::schema_for!(Envelope<LsPayload>),
    },
    Schema {
        id: TASK,
        title: TASK_TITLE,
        description: TASK_DESCRIPTION,
        schema: || schemars::schema_for!(CanonicalTask),
    },
    Schema {
        id: ADD,
        title: ADD_TITLE,
        description: ADD_DESCRIPTION,
        schema: || schemars::schema_for!(Envelope<AddReport>),
    },
    Schema {
        id: CHECK,
        title: CHECK_TITLE,
        description: CHECK_DESCRIPTION,
        schema: || schemars::schema_for!(Envelope<CheckPayload>),
    },
    Schema {
        id: SET_CHECK,
        title: SET_CHECK_TITLE,
        description: SET_CHECK_DESCRIPTION,
        schema: || schemars::schema_for!(Envelope<CheckSetPayload>),
    },
    Schema {
        id: GRAPH,
        title: GRAPH_TITLE,
        description: GRAPH_DESCRIPTION,
        schema: || schemars::schema_for!(Envelope<GraphPayload>),
    },
    Schema {
        id: PRESET_LS,
        title: PRESET_LS_TITLE,
        description: PRESET_LS_DESCRIPTION,
        schema: || schemars::schema_for!(Envelope<PresetsPayload>),
    },
    Schema {
        id: EXPLAIN,
        title: EXPLAIN_TITLE,
        description: EXPLAIN_DESCRIPTION,
        schema: || schemars::schema_for!(Envelope<ExplainIndexPayload>),
    },
    Schema {
        id: RULES,
        title: RULES_TITLE,
        description: RULES_DESCRIPTION,
        schema: || schemars::schema_for!(Envelope<RulesPayload>),
    },
    Schema {
        id: TAGS,
        title: TAGS_TITLE,
        description: TAGS_DESCRIPTION,
        schema: || schemars::schema_for!(Envelope<TagsPayload>),
    },
    Schema {
        id: EVAL,
        title: EVAL_TITLE,
        description: EVAL_DESCRIPTION,
        schema: || schemars::schema_for!(Envelope<CanonicalTask>),
    },
    Schema {
        id: BOARD,
        title: BOARD_TITLE,
        description: BOARD_DESCRIPTION,
        schema: || schemars::schema_for!(Envelope<BoardPayload>),
    },
    Schema {
        id: AT,
        title: AT_TITLE,
        description: AT_DESCRIPTION,
        schema: || schemars::schema_for!(Envelope<AtListPayload>),
    },
    Schema {
        id: AT_PROJECT,
        title: AT_PROJECT_TITLE,
        description: AT_PROJECT_DESCRIPTION,
        schema: || schemars::schema_for!(Envelope<AtProjectJson>),
    },
    Schema {
        id: AT_SET,
        title: AT_SET_TITLE,
        description: AT_SET_DESCRIPTION,
        schema: || schemars::schema_for!(Envelope<AtSetJson>),
    },
    Schema {
        id: SET_LS,
        title: SET_LS_TITLE,
        description: SET_LS_DESCRIPTION,
        schema: || schemars::schema_for!(Envelope<SetLsPayload>),
    },
    Schema {
        id: PACKS,
        title: PACKS_TITLE,
        description: PACKS_DESCRIPTION,
        schema: || schemars::schema_for!(Envelope<PacksPayload>),
    },
    Schema {
        id: DESCRIBE,
        title: DESCRIBE_TITLE,
        description: DESCRIBE_DESCRIPTION,
        schema: || schemars::schema_for!(Envelope<DescribePayload>),
    },
    Schema {
        id: VIEWS,
        title: VIEWS_TITLE,
        description: VIEWS_DESCRIPTION,
        schema: || schemars::schema_for!(Envelope<ViewsPayload>),
    },
    Schema {
        id: VIEW,
        title: VIEW_TITLE,
        description: VIEW_DESCRIPTION,
        schema: || schemars::schema_for!(Envelope<ViewSpec>),
    },
    Schema {
        id: DICTS,
        title: DICTS_TITLE,
        description: DICTS_DESCRIPTION,
        schema: || schemars::schema_for!(Envelope<DictsPayload>),
    },
    Schema {
        id: DICT,
        title: DICT_TITLE,
        description: DICT_DESCRIPTION,
        schema: || schemars::schema_for!(Envelope<DictDetailJson>),
    },
    Schema {
        id: CONFIG_SHOW,
        title: CONFIG_SHOW_TITLE,
        description: CONFIG_SHOW_DESCRIPTION,
        schema: || schemars::schema_for!(Envelope<ConfigShowPayload>),
    },
];
// jscpd:ignore-end

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

/// `schema` stamped with its `$id`, title and description as pretty JSON with
/// a trailing newline. The draft key schemars writes stays.
fn stamped(schema: schemars::Schema, id: SchemaId, title: &str, description: &str) -> String {
    let mut root = serde_json::to_value(schema).expect("a JSON schema serialises to a value");
    if let Value::Object(map) = &mut root {
        map.insert("$id".to_owned(), Value::from(id.canonical_url()));
        map.insert("title".to_owned(), Value::from(title));
        map.insert("description".to_owned(), Value::from(description));
    }
    let mut out = serde_json::to_string_pretty(&root).expect("a JSON schema serialises to JSON");
    out.push('\n');
    out
}

#[cfg(test)]
#[path = "schema_tests.rs"]
mod tests;
