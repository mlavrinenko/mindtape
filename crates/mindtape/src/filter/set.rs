//! The names a task takes from the tasks around it. Split out of `mod.rs` for
//! the line cap; read by [`Filter::reads_set`](crate::filter::Filter::reads_set).

use evalexpr::{DefaultNumericTypes, Node, Operator};

/// The names a task takes from the tasks around it: the `links:` graph through
/// `ready`, `blocked`, `blocks` and `topo`, and its children's `children` and
/// `progress`.
pub(crate) const SET_FIELDS: [&str; 6] =
    ["ready", "blocked", "blocks", "topo", "children", "progress"];

/// Whether `node` or any node under it calls `count(...)`.
pub(crate) fn calls_count(node: &Node<DefaultNumericTypes>) -> bool {
    matches!(node.operator(), Operator::FunctionIdentifier { identifier } if identifier == "count")
        || node.children().iter().any(calls_count)
}
