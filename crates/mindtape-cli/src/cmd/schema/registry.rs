//! The registry of machine outputs and the schemas they generate.

use serde_json::Value;

use super::SchemaId;
use crate::cmd::add::output::AddReport;
use crate::cmd::add::values::DescribePayload;
use crate::cmd::at::{AtListPayload, AtProjectJson, AtSetJson};
use crate::cmd::board::BoardPayload;
use crate::cmd::config::ConfigShowPayload;
use crate::cmd::explain::add::AddPayload;
use crate::cmd::explain::dicts::{DictDetailJson, DictsPayload};
use crate::cmd::explain::views::ViewsPayload;
use crate::cmd::packs::PacksPayload;
use crate::render::envelope::Envelope;
use crate::render::objects::{
    CheckPayload, CheckSetPayload, ExplainIndexPayload, GraphPayload, PresetsPayload, RulesPayload,
    TagsPayload,
};
use crate::render::output::SetLsPayload;
use crate::render::payload::LsPayload;
use mindtape::{CanonicalTask, ViewSpec};

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
pub const EXPLAIN_ADD: SchemaId = SchemaId {
    name: "explain-add",
    version: 1,
};

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

/// `mt board --json`.
pub const BOARD: SchemaId = SchemaId {
    name: "board",
    version: 1,
};

/// `mt at --json`.
pub const AT: SchemaId = SchemaId {
    name: "at",
    version: 1,
};

/// `mt at <name> --json` for a project.
pub const AT_PROJECT: SchemaId = SchemaId {
    name: "at-project",
    version: 1,
};

/// `mt at <name> --json` for a set.
pub const AT_SET: SchemaId = SchemaId {
    name: "at-set",
    version: 1,
};

/// `mt @set ls --json`.
pub const SET_LS: SchemaId = SchemaId {
    name: "set-ls",
    version: 1,
};

/// `mt packs install --json`.
pub const PACKS: SchemaId = SchemaId {
    name: "packs",
    version: 1,
};

/// `mt add --describe=json`.
pub const DESCRIBE: SchemaId = SchemaId {
    name: "describe",
    version: 2,
};

/// `mt explain views --json`.
pub const VIEWS: SchemaId = SchemaId {
    name: "views",
    version: 1,
};

/// `mt explain views <name> --json`.
pub const VIEW: SchemaId = SchemaId {
    name: "view",
    version: 1,
};

/// `mt explain dicts --json`.
pub const DICTS: SchemaId = SchemaId {
    name: "dicts",
    version: 1,
};

/// `mt explain dicts <name> --json`.
pub const DICT: SchemaId = SchemaId {
    name: "dict",
    version: 1,
};

/// `mt config show --json`.
pub const CONFIG_SHOW: SchemaId = SchemaId {
    name: "config-show",
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
const EXPLAIN_ADD_TITLE: &str = "Mindtape add hooks";
const EXPLAIN_ADD_DESCRIPTION: &str = "The object `mt explain add --json` prints: every merged `[[on.add]]` hook, its `when`, its actions and its origin.";
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
const DESCRIBE_TITLE: &str = "Mindtape template slots and add hooks";
const DESCRIBE_DESCRIPTION: &str = "The object `mt add --describe=json` prints: the template's slots, each with its kind, default and choices, and the project's `[[on.add]]` hooks.";
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
        id: EXPLAIN_ADD,
        title: EXPLAIN_ADD_TITLE,
        description: EXPLAIN_ADD_DESCRIPTION,
        schema: || schemars::schema_for!(Envelope<AddPayload>),
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
