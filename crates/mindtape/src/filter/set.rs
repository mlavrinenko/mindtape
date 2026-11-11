//! The names a task takes from the tasks around it. Split out of `mod.rs` for
//! the line cap; read by [`Filter::reads_set`](crate::filter::Filter::reads_set).

use std::collections::BTreeSet;

use crate::config::Config;
use crate::filter::Filter;

/// The names a task takes from the tasks around it: the `links:` graph through
/// `ready`, `blocked`, `blocks` and `topo`, and its children's `children` and
/// `progress`.
pub(crate) const SET_FIELDS: [&str; 6] =
    ["ready", "blocked", "blocks", "topo", "children", "progress"];

/// The functions whose answer depends on the set, not just the task. A bare
/// `ready` is a set field above; `ready()` is a call the name walk cannot see.
pub(crate) const SET_FUNCTIONS: [&str; 3] = ["ready", "blocked", "count"];

/// Whether `name` calls a function whose answer depends on the whole task set.
pub(crate) fn is_set_function(name: &str) -> bool {
    SET_FUNCTIONS.contains(&name)
}

/// Every name `source` reads, following `in_view` into the named views.
/// `read` picks the kind: [`Filter::names`] for bare identifiers, [`Filter::calls`]
/// for functions, whose call form the bare-name walk cannot see.
pub(crate) fn reads_in(
    source: &str,
    config: &Config,
    read: fn(&Filter) -> BTreeSet<String>,
) -> BTreeSet<String> {
    let mut reads = BTreeSet::new();
    let mut seen = BTreeSet::new();
    collect_reads(source, config, read, &mut reads, &mut seen);
    reads
}

/// Every function `source` calls, following `in_view` into the named views.
pub(crate) fn calls_in(source: &str, config: &Config) -> BTreeSet<String> {
    reads_in(source, config, Filter::calls)
}

fn collect_reads(
    source: &str,
    config: &Config,
    read: fn(&Filter) -> BTreeSet<String>,
    reads: &mut BTreeSet<String>,
    seen: &mut BTreeSet<String>,
) {
    let Ok(filter) = Filter::parse(source) else {
        return;
    };
    reads.extend(read(&filter));
    for view in filter.view_refs() {
        if !seen.insert(view.clone()) {
            continue;
        }
        if let Some(spec) = config.views.get(&view) {
            collect_reads(&spec.filter, config, read, reads, seen);
        }
    }
}
