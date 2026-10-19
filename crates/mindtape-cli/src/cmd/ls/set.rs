//! `mt @set ls`: one listing over every project of a set.
//!
//! Each project resolves its own listing ([`resolve_and_select`]) under its
//! own config: its default listing or `@view`, its sort, and `-n`/`--skip`
//! counted within it. The listings join in set order under one header, with
//! a `PROJECT` column first. A project that fails to open or to list prints
//! its error naming it once the rest have printed, and the run exits 1.

use anstyle::{Effects, Style};

use super::{Listing, LsArgs, effective_format, next_command_hint, print_body, resolve_and_select};
use crate::cmd::ls::rules::{broken_part, footer_sources};
use crate::diag::{Diag, Hint, Kind, Result, emit};
use crate::project_set::HeldSet;
use crate::render::color::{Painter, is_color_active};
use crate::render::output::{Envelope, Listed, OutputFormat, SetGroup, Truncation, render_set};
use crate::util;

/// One project of the set, by name, with its listing or why it has none.
struct Group {
    name: String,
    listing: Result<Listing>,
}

/// Run `ls` over every project of `held`.
///
/// # Errors
/// Refuses `--interactive`. Returns a failure naming how many projects did
/// not list, after the rest print.
pub(crate) fn run(args: &LsArgs, held: &HeldSet) -> Result<()> {
    if args.flags.interactive {
        return Err(Diag::usage("--interactive lists one project")
            .hint(Hint::as_in("name one project", one_project(held))));
    }
    let groups: Vec<Group> = util::open_set_warned(&held.members)
        .into_iter()
        .map(|member| Group {
            listing: member
                .project
                .and_then(|project| resolve_and_select(args, project)),
            name: member.name,
        })
        .collect();
    let format = effective_format(args);
    print_body(&render_groups(&groups, format)?);
    if matches!(format, OutputFormat::Table) {
        print_footer(args, held, &groups);
    }
    let total = groups.len();
    let mut failed = 0;
    for group in groups {
        if let Err(diag) = group.listing {
            failed += 1;
            emit(&diag.in_member(&group.name));
        }
    }
    if failed == 0 {
        return Ok(());
    }
    let projects = mindtape::count(total, "project");
    Err(Diag::verbatim(
        Kind::Failed,
        format!("{failed} of {projects} did not list"),
    ))
}

/// `mt @<first member> ls`, the hint for what a set cannot do.
fn one_project(held: &HeldSet) -> String {
    let first = held.members.first().map_or("name", |(name, _)| name);
    format!("mt @{first} ls --interactive")
}

/// Every group rendered in `format`, as [`render_set`] joins them.
fn render_groups(groups: &[Group], format: OutputFormat) -> Result<String> {
    let retired: Vec<Vec<String>> = groups
        .iter()
        .map(|group| {
            group
                .listing
                .as_ref()
                .map(|listing| listing.config.retired_refs())
                .unwrap_or_default()
        })
        .collect();
    let parts: Vec<SetGroup<'_>> = groups
        .iter()
        .zip(&retired)
        .map(|(group, retired)| SetGroup {
            project: &group.name,
            listed: group.listing.as_ref().map(|listing| Listed {
                tasks: &listing.tasks,
                ctx: crate::render::RenderCtx {
                    project: Some(&group.name),
                    ..listing.render_ctx()
                },
                envelope: Envelope {
                    applied: &listing.applied,
                    truncated: listing.truncated,
                    retired,
                },
            }),
        })
        .collect();
    render_set(&parts, format)
}

/// The dim lines under a set's table: where the rows came from, how many of
/// the listed projects' tasks they are and how many of their task files
/// failed to load, then, when `-n` hid rows in any
/// project, how many and the command for the next page. Nothing when no
/// project listed, since each one's error says why.
fn print_footer(args: &LsArgs, held: &HeldSet, groups: &[Group]) {
    let painter = Painter::new(is_color_active());
    let dim = Style::new().effects(Effects::DIMMED);
    let listings: Vec<&Listing> = groups
        .iter()
        .filter_map(|group| group.listing.as_ref().ok())
        .collect();
    if listings.is_empty() {
        return;
    }
    let lines = std::iter::once(footer(&listings)).chain(next_page(args, held, &listings));
    println!();
    for line in lines {
        println!("{}", painter.paint(&line, dim));
    }
}

/// `your filter · 9 of 61 tasks in 27 projects · 1 broken`: the listings'
/// source when they all share it, the rows shown of the listed projects'
/// tasks, then their task files that failed to load.
fn footer(listings: &[&Listing]) -> String {
    let mut sources = listings
        .iter()
        .map(|listing| footer_sources(&listing.applied));
    let first = sources.next().unwrap_or_default();
    let mut parts = if sources.all(|other| other == first) {
        first
    } else {
        vec!["each project's listing".to_owned()]
    };
    let shown: usize = listings.iter().map(|listing| listing.tasks.len()).sum();
    let total: usize = listings.iter().map(|listing| listing.project_total).sum();
    parts.push(format!(
        "{shown} of {} in {}",
        mindtape::count(total, "task"),
        mindtape::count(listings.len(), "project")
    ));
    let broken = listings.iter().map(|listing| listing.broken).sum();
    parts.extend(broken_part(broken));
    parts.join(" · ")
}

/// `37 more not shown · next page: mt @home ls -n 5 --skip 5`, when `-n` cut
/// any project. Every project skipped and showed the same counts, so the
/// next page is one `--skip` for all of them.
fn next_page(args: &LsArgs, held: &HeldSet, listings: &[&Listing]) -> Option<String> {
    let cuts: Vec<_> = listings
        .iter()
        .filter_map(|listing| listing.truncated)
        .collect();
    let next_skip = cuts.iter().map(Truncation::next_skip).max()?;
    let hidden: usize = cuts
        .iter()
        .map(|cut| cut.total.saturating_sub(cut.next_skip()))
        .sum();
    let next = next_command_hint(args, Some(&format!("@{}", held.name)), next_skip);
    Some(format!("{hidden} more not shown · next page: {next}"))
}
