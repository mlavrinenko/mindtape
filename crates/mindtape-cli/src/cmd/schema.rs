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
use crate::diag::{Diag, Hint, Result};
use crate::render::objects::{
    CheckPayload, CheckSetPayload, ExplainIndexPayload, GraphPayload, PresetsPayload, RulesPayload,
    TagsPayload,
};
use crate::render::envelope::Envelope;
use crate::render::output::LsPayload;
use mindtape::CanonicalTask;

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

/// One registered output schema.
#[derive(Debug)]
pub struct Schema {
    /// The output's name and version.
    pub id: SchemaId,
    /// The schema's title. The generator stamps the same text, so the tests
    /// read it back from there rather than from this field.
    #[allow(dead_code, reason = "the schema generator stamps the same text")]
    pub title: &'static str,
    /// The schema's description, kept beside the title for the same reason.
    #[allow(dead_code, reason = "the schema generator stamps the same text")]
    pub description: &'static str,
    /// The generated schema, as pretty JSON with a trailing newline.
    pub json: fn() -> String,
}

/// Every registered schema, in the order `mt schema` lists them.
pub const SCHEMAS: &[Schema] = &[
    Schema {
        id: LS,
        title: LS_TITLE,
        description: LS_DESCRIPTION,
        json: ls_json,
    },
    Schema {
        id: TASK,
        title: TASK_TITLE,
        description: TASK_DESCRIPTION,
        json: task_json,
    },
    Schema {
        id: ADD,
        title: ADD_TITLE,
        description: ADD_DESCRIPTION,
        json: add_json,
    },
    Schema {
        id: CHECK,
        title: CHECK_TITLE,
        description: CHECK_DESCRIPTION,
        json: check_json,
    },
    Schema {
        id: SET_CHECK,
        title: SET_CHECK_TITLE,
        description: SET_CHECK_DESCRIPTION,
        json: set_check_json,
    },
    Schema {
        id: GRAPH,
        title: GRAPH_TITLE,
        description: GRAPH_DESCRIPTION,
        json: graph_json,
    },
    Schema {
        id: PRESET_LS,
        title: PRESET_LS_TITLE,
        description: PRESET_LS_DESCRIPTION,
        json: preset_ls_json,
    },
    Schema {
        id: EXPLAIN,
        title: EXPLAIN_TITLE,
        description: EXPLAIN_DESCRIPTION,
        json: explain_json,
    },
    Schema {
        id: RULES,
        title: RULES_TITLE,
        description: RULES_DESCRIPTION,
        json: rules_json,
    },
    Schema {
        id: TAGS,
        title: TAGS_TITLE,
        description: TAGS_DESCRIPTION,
        json: tags_json,
    },
    Schema {
        id: EVAL,
        title: EVAL_TITLE,
        description: EVAL_DESCRIPTION,
        json: eval_json,
    },
];

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
        print!("{}", (find(name)?.json)());
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
        write_one(&path, &(schema.json)())?;
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
            Ok(recorded) if recorded == (schema.json)() => {}
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

/// `mt schema ls`: the `ls` output schema.
fn ls_json() -> String {
    stamped(
        schemars::schema_for!(Envelope<LsPayload>),
        LS,
        LS_TITLE,
        LS_DESCRIPTION,
    )
}

/// `mt schema task`: the bare canonical task record.
fn task_json() -> String {
    stamped(
        schemars::schema_for!(CanonicalTask),
        TASK,
        TASK_TITLE,
        TASK_DESCRIPTION,
    )
}

/// `mt schema check`: the `mt check --json` object.
fn check_json() -> String {
    stamped(
        schemars::schema_for!(Envelope<CheckPayload>),
        CHECK,
        CHECK_TITLE,
        CHECK_DESCRIPTION,
    )
}

/// `mt schema set-check`: the set check's object.
fn set_check_json() -> String {
    stamped(
        schemars::schema_for!(Envelope<CheckSetPayload>),
        SET_CHECK,
        SET_CHECK_TITLE,
        SET_CHECK_DESCRIPTION,
    )
}

/// `mt schema graph`: the graph object.
fn graph_json() -> String {
    stamped(
        schemars::schema_for!(Envelope<GraphPayload>),
        GRAPH,
        GRAPH_TITLE,
        GRAPH_DESCRIPTION,
    )
}

/// `mt schema preset-ls`: the preset listing object.
fn preset_ls_json() -> String {
    stamped(
        schemars::schema_for!(Envelope<PresetsPayload>),
        PRESET_LS,
        PRESET_LS_TITLE,
        PRESET_LS_DESCRIPTION,
    )
}

/// `mt schema explain`: the explain index object.
fn explain_json() -> String {
    stamped(
        schemars::schema_for!(Envelope<ExplainIndexPayload>),
        EXPLAIN,
        EXPLAIN_TITLE,
        EXPLAIN_DESCRIPTION,
    )
}

/// `mt schema rules`: the rules listing object.
fn rules_json() -> String {
    stamped(
        schemars::schema_for!(Envelope<RulesPayload>),
        RULES,
        RULES_TITLE,
        RULES_DESCRIPTION,
    )
}

/// `mt schema tags`: the tags listing object.
fn tags_json() -> String {
    stamped(
        schemars::schema_for!(Envelope<TagsPayload>),
        TAGS,
        TAGS_TITLE,
        TAGS_DESCRIPTION,
    )
}

/// `mt schema eval`: one task record inside the eval envelope.
fn eval_json() -> String {
    stamped(
        schemars::schema_for!(Envelope<CanonicalTask>),
        EVAL,
        EVAL_TITLE,
        EVAL_DESCRIPTION,
    )
}

/// `mt schema add`: the `mt add --json` report schema.
fn add_json() -> String {
    stamped(
        schemars::schema_for!(Envelope<AddReport>),
        ADD,
        ADD_TITLE,
        ADD_DESCRIPTION,
    )
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
