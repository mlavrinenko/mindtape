//! The facts a [`crate::Condition`] evaluates against.
//!
//! Split out of [`crate::filter::condition`] so that module stays the evaluator
//! and this one the domain it reads. It is the same seam
//! [`crate::config::ls_columns`] took out of [`crate::config::ls`], and for the
//! same line-cap reason.
//!
//! Two groups live here, and the difference matters:
//!
//! - Ambient facts ([`ConditionEnv`]'s own fields): the branch, the date, the
//!   process environment, the tty, the active format. Injected, never probed:
//!   [`crate::filter::condition`] touches neither the clock nor the filesystem.
//! - Listing facts ([`ColumnFacts`]): what the selected rows and the caller's
//!   column request say. These are not ambient at all. They only exist once a
//!   renderer has rows in hand, so they arrive as a sub-struct the render layer
//!   swaps per column, not as more top-level fields.

use std::collections::BTreeMap;

/// Environment facts a [`crate::Condition`] evaluates against: no task, no
/// config, just ambient project state plus the [`ColumnFacts`] of whichever
/// column the renderer resolves. The module doc explains the injection.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConditionEnv {
    /// Current git branch name, if resolvable. `None` when the project is not
    /// a git working copy, or when the repository has a detached `HEAD`.
    /// `on_branch` reads that as "no branch matches".
    pub branch: Option<String>,
    /// Today's date as `YYYY-MM-DD`, the sole input to `now()` and
    /// `weekday()`.
    pub today: String,
    /// Environment variable snapshot backing `env("NAME")`. A name absent
    /// from the map reads as `""`, same as an unset variable.
    pub env: BTreeMap<String, String>,
    /// Whether stdout is a terminal, backing `tty()`. Injected rather than
    /// probed here for the same reason as every other field: this module
    /// never touches the process or the filesystem.
    pub tty: bool,
    /// Whether stdin is a terminal, backing `stdin_tty()`. A run can have one
    /// without the other. A harness that captures output through a pipe still
    /// reads keys from a terminal. An agent terminal that shows output on a
    /// pty hands its shell `/dev/null` for input. `tty()` answers whether a
    /// human reads the output. This answers whether one can type an answer.
    pub stdin_tty: bool,
    /// The active output format, backing `format()`. Empty when the caller
    /// has no format to speak of (a condition evaluated outside a rendering
    /// command), which compares unequal to every real format name.
    pub format: String,
    /// The column the renderer resolves, backing `nonempty()`,
    /// `contested()`, `named()` and `blocked()`. All-false for a caller that
    /// resolves no column at all, such as an `[[ls.rules]]` condition picking
    /// the listing bundle. That is the same convention `format`'s empty string
    /// follows, and the thing that keeps `named()` acyclic: see
    /// [`ColumnFacts::named`].
    pub column: ColumnFacts,
}

impl ConditionEnv {
    /// A structurally valid, branch-less, dateful environment used only to
    /// dry-run a freshly parsed [`crate::Condition`]. A task-field identifier,
    /// unknown predicate, or malformed `on_branch` glob then surfaces as a
    /// load-time error rather than waiting for the rule to actually run.
    ///
    /// Every field is an arbitrary well-formed placeholder, not a real answer:
    /// the date is not "today" and [`ColumnFacts`] describes no column.
    /// Parse-time validation only needs *a* valid environment. `nonemtpy()`
    /// then fails at config load naming the column, instead of becoming a
    /// column that silently never renders.
    pub(crate) fn probe() -> Self {
        Self {
            branch: None,
            today: "1970-01-01".to_owned(),
            env: BTreeMap::new(),
            tty: false,
            stdin_tty: false,
            format: String::new(),
            column: ColumnFacts::default(),
        }
    }
}

/// What one `mt ls` column's `[ls.columns.<name>] when` decides against: three
/// facts about the column against the selected rows, and one about the listing.
///
/// These are the predicates that let a `when` expression carry the whole
/// per-column visibility policy, which then lives in the embedded floor rather
/// than in a Rust match.
// Four independent, orthogonal facts, one per predicate — the lint's advice
// (a state machine, or two-variant enums) would name states this record does
// not have and cost the predicate table its one-field-per-call shape.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ColumnFacts {
    /// `nonempty()`: at least one of this column's cells carries text.
    pub nonempty: bool,
    /// `contested()`: the rows disagree about this column's cell text. A
    /// listing of one row is never contested, since one row cannot disagree
    /// with itself. A blank counts as its own value, which makes a half-filled
    /// column contested.
    pub contested: bool,
    /// `named()`: the caller asked for this column by name (`--columns`, a
    /// view's `columns`, an `[[ls.rules]]` columns override).
    ///
    /// A property of the request, never of the outcome. The renderer fixes the
    /// render order before it decides any column's visibility, so this can
    /// never read back the decision it is an input to. An `[[ls.rules]]`
    /// condition, the one place that also chooses a `columns` override, sees
    /// the all-false default rather than a live answer. So the cascade cannot
    /// close a loop through it either.
    pub named: bool,
    /// `blocked()`: at least one listed task waits on an unmet dependency. A
    /// property of the listing, not of the column, so it reads the same for
    /// every column. It lives here because it is the `ready` column's whole
    /// `when`, and that is not [`Self::nonempty`]. A listing where every task
    /// is ready has non-blank `ready` cells and nothing worth a column.
    pub blocked: bool,
}
