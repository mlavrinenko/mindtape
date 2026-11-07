//! `mt @name check` on a set: check every member, hold each report, and
//! print them in set order, each problem naming its project.

use super::{CheckArgs, Findings, gather};
use crate::cmd::schema::SET_CHECK;
use crate::diag::{Diag, Hint, Kind, ProblemJson, Result, emit};
use crate::pre_write::with_body_rules;
use crate::project_set::{HeldSet, Member, member_load_problems_folded, open_set};
use crate::render::envelope::Envelope;
use crate::render::objects::{CheckSetPayload, ProjectCheckJson};

/// One member's report: its problems in print order, and whether it failed.
struct MemberReport {
    name: String,
    diags: Vec<Diag>,
    failed: bool,
    tasks: usize,
}

/// Run `mt check` on every member of `held`.
///
/// # Errors
/// Returns a refusal for the commit flags, and one when any member fails.
pub(super) fn run(held: &HeldSet, args: &CheckArgs) -> Result<()> {
    refuse_commit_flags(held, args)?;
    let reports = report_all(open_set(&held.members), args.json || crate::util::verbose());
    let failed = reports.iter().filter(|report| report.failed).count();
    if args.json {
        print_json(&reports)?;
    } else {
        for diag in reports.iter().flat_map(|report| &report.diags) {
            emit(diag);
        }
    }
    if failed > 0 {
        return Err(Diag::verbatim(
            Kind::Refused,
            format!(
                "{failed} of {} failed",
                mindtape::count(reports.len(), "project")
            ),
        ));
    }
    if !args.quiet && !args.json {
        let tasks: usize = reports.iter().map(|report| report.tasks).sum();
        println!(
            "ok: {}, {}",
            mindtape::count(reports.len(), "project"),
            mindtape::count(tasks, "task")
        );
    }
    Ok(())
}

/// Check every member on a thread of its own, and return the reports in the
/// order the members came. The checks print nothing and share no state, so
/// the order the threads finish in shows nowhere.
fn report_all(members: Vec<Member>, verbose: bool) -> Vec<MemberReport> {
    mindtape::map_parallel(members, |member| report(member, verbose))
}

/// `--commit-msg` and `--commits` read one repo's git.
fn refuse_commit_flags(held: &HeldSet, args: &CheckArgs) -> Result<()> {
    let flag = match (&args.commit_msg, &args.commits) {
        (Some(_), _) => "--commit-msg",
        (_, Some(_)) => "--commits",
        _ => return Ok(()),
    };
    let diag = Diag::verbatim(
        Kind::Refused,
        format!(
            "`{flag}` runs on one project, and @{} holds {}",
            held.name,
            mindtape::count(held.members.len(), "project")
        ),
    );
    Err(match held.members.first() {
        Some((first, _)) => diag.hint(Hint::as_in(
            "name one project",
            format!("mt @{first} check {flag}"),
        )),
        None => diag,
    })
}

/// Check one member, keeping what it prints instead of printing it.
fn report(member: Member, verbose: bool) -> MemberReport {
    let name = member.name;
    let project = match member.project {
        Ok(project) => project,
        Err(diag) => return failure(name, diag),
    };
    let mut diags = member_load_problems_folded(&name, &project.outcome, Kind::Failed, verbose);
    match with_body_rules(&project.config, |rules| gather(&project, rules)) {
        Ok(Findings {
            diags: found,
            problems,
            ..
        }) => {
            diags.extend(found.into_iter().map(|diag| diag.in_member(&name)));
            MemberReport {
                failed: problems > 0,
                tasks: project.outcome.tasks.len(),
                diags,
                name,
            }
        }
        Err(diag) => {
            let mut report = failure(name, diag);
            diags.append(&mut report.diags);
            report.diags = diags;
            report
        }
    }
}

fn failure(name: String, diag: Diag) -> MemberReport {
    MemberReport {
        diags: vec![diag.in_member(&name)],
        name,
        failed: true,
        tasks: 0,
    }
}

fn print_json(reports: &[MemberReport]) -> Result<()> {
    let projects: Vec<ProjectCheckJson> = reports
        .iter()
        .map(|report| ProjectCheckJson {
            project: report.name.clone(),
            ok: !report.failed,
            tasks: report.tasks,
            problems: report.diags.iter().map(ProblemJson::from).collect(),
        })
        .collect();
    let document = Envelope::new(SET_CHECK, CheckSetPayload { projects });
    println!("{}", serde_json::to_string_pretty(&document)?);
    Ok(())
}
