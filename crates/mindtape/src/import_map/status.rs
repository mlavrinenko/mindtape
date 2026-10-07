//! `status:` policy mapping: legend membership, `--map` overrides, and the
//! missing-status default.

use crate::config::Config;
use crate::date::today_iso;
use crate::model::Status;
use crate::model::canonical::CanonicalStatus;

use super::report::{MapOptions, ReportEntry};

/// Map a canonical `status`, reporting a default (missing status), a `--map`
/// rename, or an out-of-legend violation — never silently accepting or
/// inventing a value the legend does not sanction. Mirrors
/// `mindtape-cli::task_edit::initial_status`'s "must not invent" rule for the
/// missing-status case (duplicated here since the CLI depends on this crate,
/// not the reverse).
pub(crate) fn map_status(
    canonical: Option<&CanonicalStatus>,
    config: &Config,
    opts: &MapOptions,
    out: &mut Vec<ReportEntry>,
) -> Option<Status> {
    let Some(status) = canonical else {
        let initial = initial_status(config)?;
        out.push(ReportEntry::defaulted(
            "status",
            format!("no status supplied; defaulted to `{initial}`"),
        ));
        return Some(Status {
            name: initial,
            date: Some(today_iso()),
            note: None,
        });
    };
    let name = match opts.status_map.get(&status.name) {
        Some(mapped) => {
            out.push(ReportEntry::mapped(
                "status",
                format!("`{}` mapped to `{mapped}` via --map", status.name),
            ));
            mapped.clone()
        }
        None => status.name.clone(),
    };
    let legend = &config.tasks.status;
    if !legend.is_empty() && !legend.contains(&name) {
        out.push(ReportEntry::violated(
            "status",
            format!("`{name}` is not a configured status"),
        ));
    }
    let date = status.date.clone().or_else(|| {
        out.push(ReportEntry::defaulted(
            "status.date",
            "no status date supplied; defaulted to today",
        ));
        Some(today_iso())
    });
    Some(Status {
        name,
        date,
        note: status.note.clone(),
    })
}

/// The natural status for a record that supplied none: the legend's initial
/// (first non-terminal) value, falling back to the legend's first entry when
/// every entry happens to be terminal. `None` only when the legend is
/// genuinely empty — the caller must then leave the status unset rather than
/// invent one.
fn initial_status(config: &Config) -> Option<String> {
    let legend = &config.tasks.status;
    legend
        .initial()
        .or_else(|| legend.entries.first())
        .map(|entry| entry.value.clone())
}
