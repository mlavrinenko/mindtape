//! `mt check`'s template pass: every template `mt add --list` lists is filed
//! the way `mt add` files it — config-required slots spliced, the status
//! stamped — with each slot holding its `example`, else its `default`, and
//! validated as a candidate at the path `mt add` would write it to. Every
//! breach is reported at the template's file and line.
//!
//! The templates go to [`mindtape::check_candidates`] as one batch rather than
//! one call each: a candidate check validates the whole project, so a batch
//! pays for that once. Each template sits at its own stem, so no two collide.

use std::path::PathBuf;

use anyhow::{Result, anyhow};
use mindtape::{
    BodyRules, Candidate, FallbackSeverity, Severity, TemplateSources, Violation, template_failure,
};

use crate::add::{inject_header, inject_required_slots};
use crate::add_values::{list_templates, resolve_template, template_path};
use crate::task_edit::filing_status;
use crate::util::{self, Project};

/// How a breach names the built-in `task` template, which has no file.
const BUILTIN_LABEL: &str = "<built-in>/task.typ";

/// One listed template, and the task source it files, or the breaches that
/// keep it from filing one.
struct Filed {
    /// The template file, relative to the project root.
    file: PathBuf,
    /// The path `mt add` would write its task to, relative to the project root.
    rel: PathBuf,
    /// The template as authored.
    authored: String,
    /// Whether this is the built-in fallback rather than a project file.
    builtin: bool,
    /// The task source filed from the template, or its own breaches.
    filed: Result<String, Vec<Violation>>,
}

/// The breaches of every listed template, in template order.
///
/// # Errors
/// Returns an error when a listed template cannot be read, or the destination
/// `mt add` would write to cannot be resolved.
pub(crate) fn breaches(project: &Project, body_rules: BodyRules<'_>) -> Result<Vec<Violation>> {
    let dest = crate::add::dest_dir(".", None, project)?;
    let Ok(dest) = dest.strip_prefix(&project.root) else {
        // The first root lies outside the project: there is no
        // project-relative path to evaluate a candidate at, as for `mt add`.
        return Ok(Vec::new());
    };
    let mut templates = Vec::new();
    for name in list_templates(&project.config, &project.root) {
        let path = template_path(&project.config, &project.root, &name);
        let builtin = path.is_none();
        let file = path.map_or_else(
            || PathBuf::from(BUILTIN_LABEL),
            |path| {
                path.strip_prefix(&project.root)
                    .unwrap_or(&path)
                    .to_path_buf()
            },
        );
        let authored = resolve_template(&project.config, &project.root, &name)?;
        let filed = file_examples(project, &file, &authored);
        templates.push(Filed {
            rel: dest.join(format!("{name}.typ")),
            file,
            authored,
            builtin,
            filed,
        });
    }
    let fallback = project.config.templates.fallback_severity;
    Ok(judge(project, &templates, body_rules)
        .into_iter()
        .zip(&templates)
        .flat_map(|(found, template)| at_severity(found, template.builtin, fallback))
        .collect())
}

/// Each template's breaches: its own, or its filed task's checked as one
/// batch with the others.
fn judge(project: &Project, templates: &[Filed], body_rules: BodyRules<'_>) -> Vec<Vec<Violation>> {
    let candidates: Vec<Candidate<'_>> = (templates.iter())
        .filter_map(|template| {
            let source = template.filed.as_deref().ok()?;
            Some(Candidate {
                rel: &template.rel,
                source,
            })
        })
        .collect();
    let mut answers = mindtape::check_candidates(
        &project.root,
        &project.config,
        &project.outcome.tasks,
        &candidates,
        body_rules,
    )
    .into_iter();
    (templates.iter())
        .map(|template| match &template.filed {
            Err(own) => own.clone(),
            Ok(checked) => match answers.next() {
                Some(Ok(answer)) => {
                    let sources = TemplateSources {
                        file: &template.file,
                        authored: &template.authored,
                        checked,
                    };
                    mindtape::template_breaches(sources, answer.violations)
                }
                Some(Err(err)) => vec![template_failure(
                    &template.file,
                    &template.authored,
                    format!("did not load: {}", err.message),
                )],
                None => Vec::new(),
            },
        })
        .collect()
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
            severity: Some(Severity::Warn),
            ..violation
        })
        .collect()
}

/// The task source `mt add` would write from `authored` with every slot
/// holding its example, else its default; or the template's breaches when it
/// cannot be filed.
fn file_examples(
    project: &Project,
    file: &std::path::Path,
    authored: &str,
) -> Result<String, Vec<Violation>> {
    let fail = |err: anyhow::Error| vec![template_failure(file, authored, format!("{err:#}"))];
    let config = &project.config;
    let status = filing_status(config, authored, None);
    let needed = mindtape::required_core_slots(config, status.as_deref());
    let (spliced, _) = inject_required_slots(authored, &needed).map_err(fail)?;
    let unfillable = mindtape::unfillable_slots(file, authored, &spliced);
    if !unfillable.is_empty() {
        return Err(unfillable);
    }
    let filled = mindtape::fill_examples(&spliced).map_err(|err| fail(anyhow!(err.to_string())))?;
    let filled = inject_header(&filled, config, status.as_deref()).map_err(fail)?;
    Ok(util::canonical_source(config, filled))
}
