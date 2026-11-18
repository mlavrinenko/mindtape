//! The `into("status")` scanner a hook's `when` is rewritten with.
//!
//! `into` is not a filter function. [`desugar_into`] rewrites each call into a
//! filter-language constant before the expression parses. The whole filter
//! language then composes with it, and the evaluator stays unaware of it. The
//! same trick `@view` sugar plays in [`crate::filter::views`].
//!
//! [`rewrite_into`] is the one scanner behind both [`desugar_into`] (a flip's
//! constant) and a task-scope commit rule's state comparison
//! ([`crate::commit::task_expr::desugar_commit_into`]). String literals pass
//! through untouched, and the scanner recognises `into` only as a whole
//! identifier.

/// A filter atom matching every task: `id` is a plain `String` on
/// [`crate::model::Task`] and is never absent, so `has(id)` is a tautology by
/// construction. Used as the value an `into(...)` call rewrites to when the
/// transition matches.
pub(crate) const ALWAYS: &str = "has(id)";

/// The complement of [`ALWAYS`]: matches no task. What an `into(...)` call
/// rewrites to when the transition does not match.
pub(crate) const NEVER: &str = "miss(id)";

/// Rewrite every `into("name")` call in `when` into a filter-language constant:
/// [`ALWAYS`] when `name` matches the status the flip stamps, [`NEVER`]
/// otherwise. An `into` inside a string literal, or one whose call shape does
/// not parse, stays verbatim, and the filter parser then reports it as an
/// unknown function.
#[must_use]
pub fn desugar_into(when: &str, target: &str) -> String {
    rewrite_into(when, |name| {
        if name == target { ALWAYS } else { NEVER }.to_owned()
    })
}

/// Rewrite every `into("name")` call in `when` into whatever `replace` makes
/// of its name. This is the one scanner behind both [`desugar_into`] (a flip's
/// constant) and a task-scope commit rule's state comparison
/// ([`crate::commit::task_expr::desugar_commit_into`]). String literals pass
/// through untouched, and the scanner recognises `into` only as a whole
/// identifier.
#[must_use]
pub fn rewrite_into(when: &str, replace: impl Fn(&str) -> String) -> String {
    scan_call(when, "into", &replace).rewritten
}

/// Every status named by an `into(...)` call in `when`, in source order (may
/// contain duplicates). Checks a hook against the configured legend at load.
/// A status outside the legend is then one config error rather than a hook
/// that quietly never fires.
#[must_use]
pub fn into_targets(when: &str) -> Vec<String> {
    scan_call(when, "into", &|_: &str| String::new()).targets
}

/// The result of one pass over a `when` expression.
pub(crate) struct CallScan {
    /// `when` with every `<call>(...)` call replaced.
    pub(crate) rewritten: String,
    /// The names those calls referenced.
    pub(crate) targets: Vec<String>,
}

/// Single pass over `when`: rewrite `<call>("name")` calls through `replace`
/// and collect the names they reference. String literals pass through
/// untouched, and the scan recognises `call` only as a whole identifier, so
/// `zinto("x")` and `into_view("x")` pass through. Shared by `into(...)` and
/// `from_template(...)`.
pub(crate) fn scan_call(when: &str, call: &str, replace: &dyn Fn(&str) -> String) -> CallScan {
    let mut scan = CallScan {
        rewritten: String::with_capacity(when.len()),
        targets: Vec::new(),
    };
    let CallScan { rewritten, targets } = &mut scan;
    scan_expression(when, rewritten, &mut |rest, out| {
        parse_named_call(rest, call).map(|(name, len)| {
            out.push_str(&replace(&name));
            targets.push(name);
            len
        })
    });
    scan
}

/// Walk `when` once, copying string literals verbatim, and let `rewrite`
/// consume a code span at the start of `rest`: it returns the bytes consumed
/// and appends the replacement, or `None` to copy one character.
///
/// A `for` over a fixed bound, not `while cursor < when.len()`. Every mutant of
/// the cursor advance (`+=` -> `-=`/`*=`, a `copy_in_string` that returns 0, a
/// rewrite reporting length 0) otherwise spins forever, pushing another char
/// onto `rewritten` each turn. One such run, measured on 2026-09-29, reached a
/// 45 GB resident set before the host's earlyoom killed the whole sweep.
/// Correct code consumes at least one byte per turn, so the bound is never
/// reached, and a mutant returns a wrong scan.
pub(crate) fn scan_expression(
    when: &str,
    rewritten: &mut String,
    rewrite: &mut dyn FnMut(&str, &mut String) -> Option<usize>,
) {
    let mut cursor = 0;
    let mut in_string = false;
    let mut after_ident = false;
    for _ in 0..when.len().saturating_add(1) {
        let Some(rest) = when.get(cursor..) else {
            break;
        };
        let Some(ch) = rest.chars().next() else { break };
        if in_string {
            cursor += copy_in_string(rest, ch, rewritten, &mut in_string);
            continue;
        }
        if !after_ident && let Some(len) = rewrite(rest, rewritten) {
            cursor += len;
            after_ident = false;
            continue;
        }
        in_string = ch == '"';
        // `@view` is one token: the name after the sigil is not a bare word.
        after_ident = ch == '@' || is_ident_char(ch);
        rewritten.push(ch);
        cursor += ch.len_utf8();
    }
}

/// Copy one character of a string literal through verbatim, consuming the
/// character after a backslash so an escaped quote does not close the literal.
/// Returns the byte count it consumed and clears `in_string` on the closing
/// quote.
pub(crate) fn copy_in_string(
    rest: &str,
    ch: char,
    out: &mut String,
    in_string: &mut bool,
) -> usize {
    out.push(ch);
    let width = ch.len_utf8();
    if ch == '\\'
        && let Some(escaped) = rest.get(width..).and_then(|tail| tail.chars().next())
    {
        out.push(escaped);
        return width + escaped.len_utf8();
    }
    if ch == '"' {
        *in_string = false;
    }
    width
}

/// Parse a `<call>("name")` call at the start of `rest`, returning the name and
/// how many bytes the call occupies. `None` when `rest` does not open one.
pub(crate) fn parse_named_call(rest: &str, call: &str) -> Option<(String, usize)> {
    let after_name = rest.strip_prefix(call)?;
    let open = after_name.trim_start().strip_prefix('(')?;
    let literal = open.trim_start().strip_prefix('"')?;
    let (name, consumed) = literal.split_once('"')?;
    let closed = consumed.trim_start().strip_prefix(')')?;
    Some((name.to_owned(), rest.len() - closed.len()))
}

/// Whether `ch` can appear inside a filter-language identifier.
///
/// `-` joins a kebab-case field name (`x-template`) and `.` walks into a
/// record (`derived.template`). A scan for a bare name must treat both as part
/// of the word, or `x-template` would read as the `template` test.
pub(crate) fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.')
}
