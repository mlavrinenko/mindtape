//! The facts a [`crate::Condition`] evaluates against.
//!
//! Split out of [`crate::condition`] so that module stays the evaluator and
//! this one the domain it reads — the same seam [`crate::config_ls_columns`]
//! took out of [`crate::config_ls`], and for the same line-cap reason.
//!
//! Two groups live here, and the difference matters:
//!
//! - Ambient facts ([`ConditionEnv`]'s own fields) — the branch, the date, the
//!   process environment, the tty, the active format. Injected, never probed:
//!   [`crate::condition`] touches neither the clock nor the filesystem.
//! - Listing facts ([`ColumnFacts`]) — what the selected rows and the caller's
//!   column request say. These are not ambient at all; they only exist once a
//!   renderer has rows in hand, which is why they arrive as a sub-struct the
//!   render layer swaps per column rather than as more top-level fields.

use std::collections::BTreeMap;

/// Environment facts a [`crate::Condition`] evaluates against — no task, no
/// config, just ambient project state plus the [`ColumnFacts`] of whichever
/// column is being resolved. See the module doc for why this is injected.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConditionEnv {
    /// Current git branch name, if resolvable. `None` when the project is not
    /// a git working copy, or `HEAD` is detached — `on_branch` reads that as
    /// "no branch matches".
    pub branch: Option<String>,
    /// Today's date as `YYYY-MM-DD`, the sole input to `now()` and
    /// `weekday()`.
    pub today: String,
    /// Environment variable snapshot backing `env("NAME")`; a name absent
    /// from the map reads as `""`, same as an unset variable.
    pub env: BTreeMap<String, String>,
    /// Whether stdout is a terminal, backing `tty()`. Injected rather than
    /// probed here for the same reason as every other field: this module
    /// never touches the process or the filesystem.
    pub tty: bool,
    /// The active output format, backing `format()`. Empty when the caller
    /// has no format to speak of (a condition evaluated outside a rendering
    /// command), which compares unequal to every real format name.
    pub format: String,
    /// The column currently being resolved, backing `nonempty()`,
    /// `contested()`, `named()` and `blocked()`. All-false for a caller
    /// resolving no column at all — an `[[ls.rules]]` condition picking the
    /// listing bundle — the same convention `format`'s empty string follows,
    /// and the thing that keeps `named()` acyclic: see [`ColumnFacts::named`].
    pub column: ColumnFacts,
}

impl ConditionEnv {
    /// A structurally valid, branch-less, dateful environment used only to
    /// dry-run a freshly parsed [`crate::Condition`], so a task-field
    /// identifier, unknown predicate, or malformed `on_branch` glob surfaces
    /// as a load-time error rather than waiting for the rule to actually run.
    ///
    /// Every field is an arbitrary well-formed placeholder, not a real
    /// answer — the date is not "today" and [`ColumnFacts`] describes no
    /// column. Parse-time validation only needs *a* valid environment; what it
    /// buys is that `nonemtpy()` fails at config load naming the column,
    /// instead of becoming a column that silently never renders.
    pub(crate) fn probe() -> Self {
        Self {
            branch: None,
            today: "1970-01-01".to_owned(),
            env: BTreeMap::new(),
            tty: false,
            format: String::new(),
            column: ColumnFacts::default(),
        }
    }
}

/// What one `mt ls` column's `[ls.columns.<name>] when` decides against: three
/// facts about the column against the selected rows, and one about the listing.
///
/// These are the predicates that let a `when` expression say everything the
/// render layer used to decide for itself, which is what moved the per-column
/// visibility policy out of a Rust match and into the embedded floor.
// Four independent, orthogonal facts, one per predicate — the lint's advice
// (a state machine, or two-variant enums) would name states this record does
// not have and cost the predicate table its one-field-per-call shape.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ColumnFacts {
    /// `nonempty()`: at least one of this column's cells carries text.
    pub nonempty: bool,
    /// `contested()`: the rows disagree about this column's cell text. A
    /// listing of one row is never contested — one row cannot disagree with
    /// itself — and a blank counts as its own value, so a half-filled column
    /// is contested.
    pub contested: bool,
    /// `named()`: the caller asked for this column by name (`--columns`, a
    /// view's `columns`, an `[[ls.rules]]` columns override).
    ///
    /// A property of the REQUEST, never of the outcome. The render order is
    /// fixed before any column's visibility is decided, so this can never read
    /// back the decision it is an input to; and an `[[ls.rules]]` condition —
    /// the one place a `columns` override is itself chosen — sees the
    /// all-false default rather than a live answer, so the cascade cannot
    /// close a loop through it either.
    pub named: bool,
    /// `blocked()`: at least one listed task is blocked on an unmet
    /// dependency. A property of the listing, not of the column, so it reads
    /// the same for every column — it lives here because it is the `ready`
    /// column's whole `when`, and that is not [`Self::nonempty`]: a listing
    /// where every task is ready has non-blank `ready` cells and nothing worth
    /// a column.
    pub blocked: bool,
}
