//! The project's shape, read off the config without loading a task.
//!
//! `mt explain` prints this as its `project:`, `task:` and `commits:` blocks.
//! Every field comes from the merged config. The answer costs one config load
//! and one template-directory scan. The code doesn't load a task, count
//! anything or evaluate a filter. A key the project leaves open stays out of
//! its block, and [`Shape::absent`] names it once.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;

use crate::config::Requirement;
use crate::config::add_hooks::template_names;
use crate::link::kind_names;
use crate::{ActorKind, CommentsPolicy, Config, Input, InputKind, NumericField};

/// The `project:`, `task:` and `commits:` blocks, and the keys they leave out.
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    /// Where tasks live, and who works them.
    pub project: ProjectShape,
    /// The fields a task carries.
    pub task: TaskShape,
    /// How a commit references its task.
    pub commits: CommitsShape,
    /// The keys the blocks leave out, because the project leaves them open.
    pub absent: Vec<String>,
}

/// The `project:` block of the index: where tasks live, and who works them.
#[derive(Debug, Clone, PartialEq, Serialize, schemars::JsonSchema)]
pub struct ProjectShape {
    /// Each task root, with the tag it emits.
    pub roots: Vec<RootShape>,
    /// The status legend grouped by `terminal`, or `null` when the project
    /// clears it.
    pub statuses: Option<StatusShape>,
    /// The actor registry, in declaration order.
    pub actors: Vec<ActorShape>,
    /// The `[rm] default-filter`, or `null` when unset.
    pub rm: Option<String>,
}

/// One task root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct RootShape {
    /// The root directory, relative to the project root.
    pub path: String,
    /// The tag the root emits.
    pub tag: String,
}

/// The status legend, grouped by whether a status ends a task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct StatusShape {
    /// The statuses that leave a task open, in legend order.
    pub open: Vec<String>,
    /// The statuses that end a task, in legend order.
    pub terminal: Vec<String>,
}

/// One actor in the registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct ActorShape {
    /// The actor's name.
    pub name: String,
    /// `human` or `llm`.
    pub kind: String,
    /// The roles the actor holds.
    pub roles: Vec<String>,
}

/// The `task:` block of the index: the fields a task carries.
#[derive(Debug, Clone, PartialEq, Serialize, schemars::JsonSchema)]
pub struct TaskShape {
    /// The priority scale, and the framework that computes it.
    pub priority: PriorityShape,
    /// The difficulty scale.
    pub difficulty: ScaleShape,
    /// The estimate units, or `null` when any unit is legal.
    pub estimate: Option<EstimateShape>,
    /// The allowed `kind` values, empty when any is legal.
    pub kinds: Vec<String>,
    /// Every link kind the project accepts.
    pub links: Vec<String>,
    /// The `[tasks.comments] policy`, or `null` when it allows comments.
    pub comments: Option<String>,
    /// Each `x-` field the config declares or a rule reads.
    pub extensions: Vec<ExtensionShape>,
    /// Every template `mt add` accepts, and the default.
    pub templates: TemplateShape,
}

/// The priority scale, and the framework that computes a score on it.
#[derive(Debug, Clone, PartialEq, Serialize, schemars::JsonSchema)]
pub struct PriorityShape {
    /// The framework that computes priority, or `null` for a raw number.
    pub framework: Option<String>,
    /// The framework's inputs, empty for a raw number.
    pub inputs: Vec<InputShape>,
    /// The scale's inclusive minimum.
    pub min: f64,
    /// The scale's inclusive maximum.
    pub max: f64,
}

/// One input a framework scores from, with what it accepts.
#[derive(Debug, Clone, PartialEq, Serialize, schemars::JsonSchema)]
pub struct InputShape {
    /// The input's name, as `score` and each category's `when` refer to it.
    pub name: String,
    /// `number`, `bool` or `enum`.
    pub kind: String,
    /// The inclusive lower bound of a `number` input, `null` otherwise.
    pub min: Option<f64>,
    /// The inclusive upper bound of a `number` input, `null` otherwise.
    pub max: Option<f64>,
    /// The allowed values of an `enum` input, empty otherwise.
    pub values: Vec<String>,
}

/// A numeric scale's inclusive bounds.
#[derive(Debug, Clone, PartialEq, Serialize, schemars::JsonSchema)]
pub struct ScaleShape {
    /// The scale's inclusive minimum.
    pub min: f64,
    /// The scale's inclusive maximum.
    pub max: f64,
}

/// The estimate units, and whether a task must carry one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct EstimateShape {
    /// The legal units.
    pub units: Vec<String>,
    /// The `required` filter: `false`, `true`, or an expression.
    pub required: String,
}

/// One `x-` field, and the rules that read it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct ExtensionShape {
    /// The field's name, such as `x-goals`.
    pub name: String,
    /// The rules that read it, by name.
    pub rules: Vec<String>,
}

/// Every template `mt add` accepts, and the one it picks by default.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct TemplateShape {
    /// The template stems, sorted.
    pub names: Vec<String>,
    /// The template `mt add` uses when the caller gives none.
    pub default: String,
}

/// The `commits:` block of the index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct CommitsShape {
    /// The trailer key whose values reference tasks.
    pub trailer: String,
}

/// The project's shape, read off the config alone. `root` is the project root
/// the template directories resolve against.
#[must_use]
pub fn shape(config: &Config, root: &Path) -> Shape {
    let project = project_shape(config);
    let task = task_shape(config, root);
    let commits = CommitsShape {
        trailer: config.commits.trailer.clone(),
    };
    let absent = absent(&project, &task);
    Shape {
        project,
        task,
        commits,
        absent,
    }
}

/// The `project:` block: the roots, the status legend, the actors and the
/// `[rm] default-filter`.
fn project_shape(config: &Config) -> ProjectShape {
    let legend = &config.tasks.status;
    ProjectShape {
        roots: config
            .tasks
            .roots
            .iter()
            .map(|root| RootShape {
                path: root.path().to_owned(),
                tag: root.name(),
            })
            .collect(),
        statuses: (!legend.is_empty()).then(|| StatusShape {
            open: legend
                .entries
                .iter()
                .filter(|entry| !entry.terminal)
                .map(|entry| entry.value.clone())
                .collect(),
            terminal: legend.terminal(),
        }),
        actors: config
            .actors
            .iter()
            .map(|actor| ActorShape {
                name: actor.name.clone(),
                kind: match actor.kind {
                    ActorKind::Human => "human".to_owned(),
                    ActorKind::Llm => "llm".to_owned(),
                },
                roles: actor.roles.clone(),
            })
            .collect(),
        rm: config
            .rm
            .default_filter
            .clone()
            .filter(|filter| !filter.trim().is_empty()),
    }
}

/// The `task:` block: the scales, the vocabularies and the templates.
fn task_shape(config: &Config, root: &Path) -> TaskShape {
    let priority = &config.tasks.priority;
    let framework = priority
        .framework
        .as_ref()
        .and_then(|name| priority.frameworks.get(name));
    TaskShape {
        priority: PriorityShape {
            framework: priority.framework.clone(),
            inputs: framework
                .map(|framework| {
                    framework
                        .inputs
                        .iter()
                        .map(|(name, input)| input_shape(name, input))
                        .collect()
                })
                .unwrap_or_default(),
            min: priority.field.min,
            max: priority.field.max,
        },
        difficulty: scale(&config.tasks.difficulty),
        estimate: estimate_shape(config),
        kinds: config
            .tasks
            .fields
            .get("kind")
            .map(|field| field.values.clone())
            .unwrap_or_default(),
        links: kind_names(&config.tasks.links)
            .into_iter()
            .map(str::to_owned)
            .collect(),
        comments: comments_shape(config),
        extensions: extensions(config),
        templates: TemplateShape {
            names: template_names(config, root),
            default: config.templates.default.clone(),
        },
    }
}

/// A numeric field's inclusive bounds.
fn scale(field: &NumericField) -> ScaleShape {
    ScaleShape {
        min: field.min,
        max: field.max,
    }
}

/// One framework input, with the bounds or values it accepts.
fn input_shape(name: &str, input: &Input) -> InputShape {
    InputShape {
        name: name.to_owned(),
        kind: match input.kind {
            InputKind::Number => "number",
            InputKind::Bool => "bool",
            InputKind::Enum => "enum",
        }
        .to_owned(),
        min: input.min,
        max: input.max,
        values: input.values.clone(),
    }
}

/// The estimate units, or `None` when any unit is legal.
fn estimate_shape(config: &Config) -> Option<EstimateShape> {
    let estimate = &config.tasks.estimate;
    (!estimate.units.is_empty()).then(|| EstimateShape {
        units: estimate.units.clone(),
        required: requirement(&estimate.required),
    })
}

/// A `required` value as the filter it is: `false`, `true`, or an expression.
fn requirement(required: &Requirement) -> String {
    match required {
        Requirement::No => "false".to_owned(),
        Requirement::Always => "true".to_owned(),
        Requirement::Expr(expr) => expr.clone(),
    }
}

/// The comment policy, or `None` when the project allows comments.
fn comments_shape(config: &Config) -> Option<String> {
    match config.tasks.comments.policy {
        CommentsPolicy::Allow => None,
        CommentsPolicy::Warn => Some("warned".to_owned()),
        CommentsPolicy::Forbid => Some("forbidden".to_owned()),
    }
}

/// Every `x-` field the config declares or a `check` rule reads, with the
/// rules that read it.
fn extensions(config: &Config) -> Vec<ExtensionShape> {
    let mut names: BTreeMap<String, Vec<String>> = config
        .tasks
        .fields
        .keys()
        .filter(|name| name.starts_with("x-"))
        .map(|name| (name.clone(), Vec::new()))
        .collect();
    for (index, rule) in config.tasks.rules.iter().enumerate() {
        let Some(field) = rule
            .check
            .as_ref()
            .and_then(|check| check.field.as_deref())
            .filter(|field| field.starts_with("x-"))
        else {
            continue;
        };
        let label = rule.name.clone().unwrap_or_else(|| format!("[{index}]"));
        names.entry(field.to_owned()).or_default().push(label);
    }
    names
        .into_iter()
        .map(|(name, rules)| ExtensionShape { name, rules })
        .collect()
}

/// The keys the blocks leave out, because the project leaves them open.
fn absent(project: &ProjectShape, task: &TaskShape) -> Vec<String> {
    [
        project.statuses.is_none().then_some("statuses"),
        project.actors.is_empty().then_some("actors"),
        project.rm.is_none().then_some("rm"),
        task.estimate.is_none().then_some("estimate"),
        task.kinds.is_empty().then_some("kinds"),
        task.comments.is_none().then_some("comments"),
        task.extensions.is_empty().then_some("extensions"),
    ]
    .into_iter()
    .flatten()
    .map(str::to_owned)
    .collect()
}
