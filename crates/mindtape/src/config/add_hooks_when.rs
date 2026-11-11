//! The `template` forms an `[[on.add]]` `when` may use, rewritten into
//! filter-language constants so the evaluator never learns them: `template ==
//! "name"`, `template != "name"` and `from_template("name")`.
//!
//! [`add_hooks`](crate::config::add_hooks) re-exports [`desugar_add_when`] and
//! [`when_templates`], its public surface.

use crate::config::hooks::{ALWAYS, NEVER, is_ident_char, scan_call, scan_expression};

/// Rewrite a hook's `when` for the template `mt add` used: every
/// `from_template("name")` and `template ==` / `template !=` test becomes a
/// filter-language constant, [`ALWAYS`] or [`NEVER`].
///
/// # Errors
/// Returns a message when `when` uses `template` bare, compares it with
/// another operator, or compares it with something other than a string literal.
pub fn desugar_add_when(when: &str, template: &str) -> Result<String, String> {
    rewrite_when(when, template).map(|(rewritten, _)| rewritten)
}

/// Every template name `when` references, in source order, deduplicated.
///
/// # Errors
/// The same malformed `when` forms [`desugar_add_when`] refuses.
pub fn when_templates(when: &str) -> Result<Vec<String>, String> {
    let (_, names) = rewrite_when(when, "")?;
    let mut seen = std::collections::BTreeSet::new();
    let mut unique = Vec::new();
    for name in names {
        if seen.insert(name.clone()) {
            unique.push(name);
        }
    }
    Ok(unique)
}

/// [`desugar_add_when`] plus the names it saw, before deduplication.
fn rewrite_when(when: &str, template: &str) -> Result<(String, Vec<String>), String> {
    let calls = scan_call(when, "from_template", &|name: &str| {
        constant(name == template).to_owned()
    });
    let compared = scan_comparisons(&calls.rewritten, template)?;
    let mut names = calls.targets;
    names.extend(compared.names);
    Ok((compared.rewritten, names))
}

/// [`ALWAYS`] when `matched`, else [`NEVER`].
fn constant(matched: bool) -> &'static str {
    if matched { ALWAYS } else { NEVER }
}

/// One `template == "name"` or `template != "name"` test.
struct Comparison {
    name: String,
    negated: bool,
    /// Bytes the test occupies, including any whitespace around the operator.
    len: usize,
}

/// The result of rewriting every comparison in one `when`.
struct ComparisonScan {
    rewritten: String,
    names: Vec<String>,
}

/// Rewrite every `template == "name"` / `template != "name"` test in `when`
/// into a constant, leaving the rest of the expression alone. Any other use of
/// `template` — bare, another operator, or a non-literal name — is an error.
fn scan_comparisons(when: &str, template: &str) -> Result<ComparisonScan, String> {
    let mut scan = ComparisonScan {
        rewritten: String::with_capacity(when.len()),
        names: Vec::new(),
    };
    let mut error = None;
    let ComparisonScan { rewritten, names } = &mut scan;
    scan_expression(when, rewritten, &mut |rest, out| {
        if error.is_some() {
            return None;
        }
        match parse_template_comparison(rest) {
            Ok(Some(comparison)) => {
                let matched = (comparison.name == template) != comparison.negated;
                out.push_str(constant(matched));
                names.push(comparison.name);
                Some(comparison.len)
            }
            Err(message) => {
                error = Some(message);
                None
            }
            Ok(None) => {
                if is_word(rest, "from_template") {
                    error = Some(
                        "from_template(...) takes a template name in double quotes".to_owned(),
                    );
                } else if is_word(rest, "template") {
                    error = Some(
                        "bare `template` is not a condition: compare it with == or !=, \
                         or write from_template(\"name\")"
                            .to_owned(),
                    );
                }
                None
            }
        }
    });
    match error {
        Some(message) => Err(message),
        None => Ok(scan),
    }
}

/// Parse a `template` comparison at the start of `rest`, if one opens there.
fn parse_template_comparison(rest: &str) -> Result<Option<Comparison>, String> {
    let Some(after) = rest.strip_prefix("template") else {
        return Ok(None);
    };
    if after.chars().next().is_some_and(is_ident_char) {
        return Ok(None);
    }
    let operator = after.trim_start();
    let (negated, tail) = if let Some(tail) = operator.strip_prefix("==") {
        (false, tail)
    } else if let Some(tail) = operator.strip_prefix("!=") {
        (true, tail)
    } else if operator.starts_with('<') || operator.starts_with('>') || operator.starts_with('=') {
        return Err("only == and != compare template".to_owned());
    } else {
        return Ok(None);
    };
    let Some(literal) = tail.trim_start().strip_prefix('"') else {
        return Err("template compares with a string literal".to_owned());
    };
    let Some(end) = literal.find('"') else {
        return Err("template compares with a string literal".to_owned());
    };
    let Some(name) = literal.get(..end) else {
        return Err("template compares with a string literal".to_owned());
    };
    let Some(consumed) = literal.get(end + 1..) else {
        return Err("template compares with a string literal".to_owned());
    };
    Ok(Some(Comparison {
        name: name.to_owned(),
        negated,
        len: rest.len() - consumed.len(),
    }))
}

/// Whether `rest` opens with `word` as a whole identifier.
fn is_word(rest: &str, word: &str) -> bool {
    let Some(after) = rest.strip_prefix(word) else {
        return false;
    };
    after.chars().next().is_none_or(|next| !is_ident_char(next))
}
