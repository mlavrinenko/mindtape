//! `mt check`'s template pass: every template `mt add --list` lists is filed
//! the way `mt add` files it — config-required slots spliced, the status
//! stamped — with each slot holding its `example`, else its `default`, and
//! validated as a candidate at the path `mt add` would write it to. Every
//! breach is reported at the template's file and line.
//!
//! Each template is checked alone, against an empty backlog, at the stem
//! `mt add` would mint from its example title: the backlog's state, and a task
//! named like a template, cannot fail a template. A breach that reads the whole
//! task set ([`mindtape::reads_set_breach`]) is one the template cannot fix, so
//! it is dropped.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::diag::Result;
use mindtape::{
    BodyRules, Candidate, FallbackSeverity, LoadError, RawValue, Severity, TaskCallSpans,
    TemplateHome, TemplateSources, Violation, template_failure,
};

use crate::cmd::add::values::{list_templates, resolve_template, template_path};
use crate::cmd::add::{inject_header, minted_stem, splice_config_slots};
use crate::cmd::install::hint::{names_prelude_package, prelude_diag, unsupported_prelude_diag};
use crate::diag::{Diag, Kind};
use crate::task_edit::filing_status;
use crate::util::{self, Project};

/// One listed template, and the task source it files, or the breaches that
/// keep it from filing one.
struct Filed {
    /// The template file relative to the project root, or `None` for the
    /// built-in fallback, which has none.
    file: Option<PathBuf>,
    /// The template's name, as `mt add -t` takes it.
    name: String,
    /// The path `mt add` would write its task to, relative to the project root.
    rel: PathBuf,
    /// The template as authored.
    authored: String,
    /// The task source filed from the template, or its own breaches.
    filed: Result<String, Vec<Violation>>,
}

impl Filed {
    /// Where a breach of this template is placed.
    fn home(&self) -> TemplateHome<'_> {
        self.file
            .as_deref()
            .map_or(TemplateHome::Builtin(&self.name), TemplateHome::File)
    }
}

/// A template's load failure, with the one diagnostic a repeated prelude
/// failure folds into: a key that groups the repeats, and the replacement.
type Failure = (Option<(String, Diag)>, Diag);

/// What the template pass found: the breaches of every listed template, in
/// template order, and the templates that did not load at all, each already a
/// [`Diag`] placed as a task file's load error is.
pub(crate) struct Found {
    /// Breaches of templates that loaded or could not be filed.
    pub violations: Vec<Violation>,
    /// One failure per project template that did not evaluate.
    pub load_failures: Vec<Diag>,
    /// The same for a built-in template at a `[templates] fallback-severity`
    /// below `error`: reported, but not a failure.
    pub load_warnings: Vec<Diag>,
}

/// The breaches of every listed template, in template order.
///
/// # Errors
/// Returns an error when a listed template cannot be read, or the destination
/// `mt add` would write to cannot be resolved.
pub(crate) fn breaches(project: &Project, body_rules: BodyRules<'_>) -> Result<Found> {
    let dest = crate::cmd::add::dest_dir(".", None, project)?;
    let Ok(dest) = dest.strip_prefix(&project.root) else {
        // The first root lies outside the project: there is no
        // project-relative path to evaluate a candidate at, as for `mt add`.
        return Ok(Found {
            violations: Vec::new(),
            load_failures: Vec::new(),
            load_warnings: Vec::new(),
        });
    };
    let mut templates = Vec::new();
    for name in list_templates(&project.config, &project.root) {
        let path = template_path(&project.config, &project.root, &name);
        let file = path.map(|path| {
            path.strip_prefix(&project.root)
                .unwrap_or(&path)
                .to_path_buf()
        });
        let authored = resolve_template(&project.config, &project.root, &name)?;
        let mut template = Filed {
            rel: dest.join(format!(
                "{}.typ",
                candidate_stem(project, &name, &authored)?
            )),
            file,
            name,
            authored,
            filed: Ok(String::new()),
        };
        template.filed = file_examples(project, template.home(), &template.authored);
        templates.push(template);
    }
    let fallback = project.config.templates.fallback_severity;
    let (judged, load_failures) = judge(project, &templates, body_rules);
    let (load_warnings, load_failures) = load_failures
        .into_iter()
        .partition(|(_, diag)| diag.kind() == Kind::Warning);
    Ok(Found {
        violations: judged
            .into_iter()
            .zip(&templates)
            .flat_map(|(mut found, template)| {
                found.extend(mindtape::code_span_examples(
                    template.home(),
                    &template.authored,
                ));
                at_severity(found, template.file.is_none(), fallback)
            })
            .collect(),
        load_failures: one_prelude_failure(load_failures, &already_said(project)),
        load_warnings: one_prelude_failure(load_warnings, &already_said(project)),
    })
}

/// The prelude failures the project load already printed, so the template pass
/// does not repeat one for the same unsupported version.
fn already_said(project: &Project) -> BTreeSet<String> {
    project
        .outcome
        .errors
        .iter()
        .filter_map(|error| {
            mindtape::unsupported_prelude_version(&error.message)
                .map(|version| format!("unsupported:{version}"))
        })
        .collect()
}

/// `failures` with each prelude problem said once: a missing package once,
/// and an unsupported version once per version, however many templates import
/// it and however many task files already named it.
fn one_prelude_failure(failures: Vec<Failure>, said: &BTreeSet<String>) -> Vec<Diag> {
    let mut said = said.clone();
    failures
        .into_iter()
        .filter_map(|(fold, diag)| match fold {
            Some((key, replacement)) => said.insert(key).then_some(replacement),
            None => Some(diag),
        })
        .collect()
}

/// The fold a prelude load failure asks for, or `None` for a failure that is
/// its own.
fn prelude_failure(template: &Filed, err: &LoadError, kind: Kind) -> Option<(String, Diag)> {
    if names_prelude_package(&err.message) {
        return Some(("missing".to_owned(), prelude_diag(kind, &err.message)));
    }
    let version = mindtape::unsupported_prelude_version(&err.message)?;
    let label = template_label(template);
    Some((
        format!("unsupported:{version}"),
        unsupported_prelude_diag(kind, version, Path::new(&label)),
    ))
}

/// The stem `mt add` mints for `name` from its title slot's example, else its
/// default; the template's own name when it has no such title.
fn candidate_stem(project: &Project, name: &str, authored: &str) -> Result<String> {
    let title = mindtape::discover_slots(authored)
        .into_iter()
        .find(|slot| slot.name == "title")
        .and_then(|slot| slot.example_string.or(slot.default_string));
    let Some(title) = title else {
        return Ok(name.to_owned());
    };
    let values = BTreeMap::from([("title".to_owned(), RawValue::Str(title))]);
    minted_stem(&values, &project.config).or_else(|_| Ok(name.to_owned()))
}

/// Each template's breaches: its own, or its filed task's checked alone against
/// no other task, less the breaches that read the whole task set. A template
/// that did not evaluate is a load failure instead, tagged with the message
/// when it names the missing prelude.
fn judge(
    project: &Project,
    templates: &[Filed],
    body_rules: BodyRules<'_>,
) -> (Vec<Vec<Violation>>, Vec<Failure>) {
    let mut failures = Vec::new();
    let judged = (templates.iter())
        .map(|template| match &template.filed {
            Err(own) => own.clone(),
            Ok(checked) => match check_alone(project, template, checked, body_rules) {
                Ok(found) => found,
                Err(err) => {
                    let kind = failure_kind(template, project.config.templates.fallback_severity);
                    let fold = prelude_failure(template, &err, kind);
                    failures.push((fold, load_failure(template, checked, &err, kind)));
                    Vec::new()
                }
            },
        })
        .collect();
    (judged, failures)
}

/// How a template that did not evaluate counts: a failure, but a warning for a
/// built-in template while `[templates] fallback-severity` is below `error`.
fn failure_kind(template: &Filed, fallback: FallbackSeverity) -> Kind {
    if template.file.is_none() && fallback != FallbackSeverity::Error {
        Kind::Warning
    } else {
        Kind::Failed
    }
}

/// The error for a template that did not evaluate, placed in the template's
/// own file: at the filled source's line and column when the template holds
/// that very line, else at its `task.with(…)` call.
fn load_failure(template: &Filed, checked: &str, err: &LoadError, kind: Kind) -> Diag {
    let diag = Diag::verbatim(kind, &err.message).typst_hints(err.hints.iter().cloned());
    let label = template_label(template);
    let spans = TaskCallSpans::parse(&template.authored);
    let line = err
        .line
        .and_then(|line| spans.relocate(&template.authored, checked, "", line));
    let Some(line) = line else {
        return diag.at_file(Path::new(&label));
    };
    let same_line = err.line.is_some_and(|checked_line| {
        let wanted = checked.lines().nth(checked_line.saturating_sub(1));
        let found = template.authored.lines().nth(line.saturating_sub(1));
        wanted
            .zip(found)
            .is_some_and(|(wanted, found)| wanted == found)
    });
    diag.at(Path::new(&label), line, err.column.filter(|_| same_line))
}

/// How a template that did not load is named: its file, or the built-in
/// template's name.
fn template_label(template: &Filed) -> String {
    template.file.as_ref().map_or_else(
        || format!("built-in template {}", template.name),
        |file| file.to_string_lossy().into_owned(),
    )
}

/// The breaches of one filed template, checked with no other task around it,
/// or why it did not load.
fn check_alone(
    project: &Project,
    template: &Filed,
    checked: &str,
    body_rules: BodyRules<'_>,
) -> Result<Vec<Violation>, LoadError> {
    let candidate = Candidate {
        rel: &template.rel,
        source: checked,
    };
    // A template's sample task is no one's work: it meets an empty change set,
    // so a rule such as `changed(...) <= estimate.src-files` never fires on it.
    let config = project.config.with_empty_change_set();
    let answer = mindtape::check_candidates(&project.root, &config, &[], &[candidate], body_rules)
        .into_iter()
        .next();
    match answer {
        Some(Ok(answer)) => {
            let sources = TemplateSources {
                home: template.home(),
                authored: &template.authored,
                checked,
            };
            let own = (answer.violations.into_iter())
                .filter(|violation| !mindtape::reads_set_breach(&config, violation))
                .collect();
            Ok(mindtape::template_breaches(sources, own))
        }
        Some(Err(err)) => Err(err),
        None => Ok(Vec::new()),
    }
}

/// A built-in template's breaches at `[templates] fallback-severity`; a
/// project template's as they are.
fn at_severity(found: Vec<Violation>, builtin: bool, fallback: FallbackSeverity) -> Vec<Violation> {
    if !builtin || fallback == FallbackSeverity::Error {
        return found;
    }
    found
        .into_iter()
        .map(|violation| Violation {
            severity: Severity::Warn,
            ..violation
        })
        .collect()
}

/// The task source `mt add` would write from `authored` with every slot
/// holding its example, else its default; or the template's breaches when it
/// cannot be filed.
fn file_examples(
    project: &Project,
    home: TemplateHome<'_>,
    authored: &str,
) -> Result<String, Vec<Violation>> {
    let fail = |err: String| vec![template_failure(home, authored, err)];
    let config = &project.config;
    let status = filing_status(config, authored, None);
    let (spliced, injected) = splice_config_slots(authored, config, status.as_deref())
        .map_err(|err| fail(format!("{err:#}")))?;
    let unfillable = mindtape::unfillable_slots(home, authored, &spliced, &injected);
    if !unfillable.is_empty() {
        return Err(unfillable);
    }
    let filled = mindtape::fill_examples(&spliced).map_err(|err| fail(err.to_string()))?;
    let filled = inject_header(&filled, config, status.as_deref())
        .map_err(|err| fail(format!("{err:#}")))?;
    Ok(util::canonical_source(config, filled))
}
