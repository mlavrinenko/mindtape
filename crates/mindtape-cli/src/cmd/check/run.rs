//! The `[commits] run` command: the project's own message check over a
//! commit's message, and report each line it prints.
//!
//! The command is the project's policy, not `Mindtape`'s. This binary only
//! knows that a non-zero exit means the message failed and that the output is
//! already worded for a reader. The same command runs over the message under
//! construction in `mt check --commit-msg` and over each past message under
//! `mt check --commits`.

use std::path::Path;

use mindtape::shell::{self, ShellOutput};

use crate::cmd::check::commit::in_commit;
use crate::diag::{Diag, Kind, Note, Result};
use crate::util;

/// Run the configured message command over `message`. `commit` is the id and
/// subject when the message comes from a range. Returns how many problems the
/// checker reported. It returns zero when the project defines no command, and
/// zero when the command passes.
///
/// # Errors
/// Fails when the command cannot start. A command that runs and exits non-zero
/// is a finding, not an error.
pub(crate) fn check(
    project: &util::Project,
    message: &str,
    commit: Option<(&str, &str)>,
) -> Result<usize> {
    let Some(command) = &project.config.commits.run else {
        return Ok(0);
    };
    if command.is_blank() {
        return Ok(0);
    }
    let output =
        shell::run_with_input(command, &project.root, &[], message.as_bytes()).map_err(|err| {
            Diag::failed("can't run commits.run")
                .value(command.to_string())
                .io(&err)
        })?;
    if output.code == Some(0) {
        return Ok(0);
    }

    let hits = hits(&output);
    if hits.is_empty() {
        let diag = Diag::refused("commit check failed")
            .value(command.to_string())
            .exit(output.code);
        crate::diag::emit(&in_commit(diag, commit));
        return Ok(1);
    }
    for hit in &hits {
        let mut diag = Diag::verbatim(Kind::Refused, hit.as_str()).rule("commits.run");
        if let Some((short, _)) = commit {
            diag = diag.at_file(Path::new(short));
        }
        crate::diag::emit(&diag);
    }
    if let Some((_, subject)) = commit {
        crate::diag::emit_note(&Note::new("commit").value(subject));
    }
    Ok(hits.len())
}

/// Every non-blank line the checker printed, stdout first, in order.
fn hits(output: &ShellOutput) -> Vec<String> {
    output
        .stdout
        .lines()
        .chain(output.stderr.lines())
        .map(str::trim_end)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
#[path = "run_tests.rs"]
mod tests;
