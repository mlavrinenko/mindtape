//! The `key OP value` token grammar behind both `--set` and the bare
//! arguments after `--`.
//!
//! Three operators over one shape. `=` sets a slot, `+=` appends to its list,
//! and `@` reads the value out of a file. The two carriers parse identically
//! on purpose: `--set tags+=docs` and a trailing `tags+=docs` are the same
//! token reaching this parser, so the shorthand can never drift into a second
//! grammar with its own rules.
//!
//! Split out of `cmd/add/values.rs` — which is about where values come FROM
//! (`-f` sources, the environment, the template's defaults) — so the grammar
//! itself has one home to read, and the sources file keeps room to grow.

use std::collections::BTreeMap;

use mindtape::RawValue;

use crate::diag::{Diag, Hint};

/// Which operator a token carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    /// `=` — the value is the rest of the token.
    Set,
    /// `@` — the value is the contents of the file the rest of the token names.
    File,
}

/// Apply one `key OP value` token to the fill map.
///
/// # Errors
/// Returns an error when the token carries no operator, tries to append a
/// file, or names a file that cannot be read.
pub fn apply_token(out: &mut BTreeMap<String, RawValue>, token: &str) -> Result<(), Diag> {
    let (key, op, rest) = split_token(token)?;
    let append = key.ends_with('+');
    let name = key.strip_suffix('+').unwrap_or(key);
    if append && op == Op::File {
        return Err(Diag::usage("+@ is not an operator")
            .value(token)
            .hint(Hint::new(
                "@ reads one file into one value; pass a list through -f as a YAML or JSON array",
            )));
    }
    let value = match op {
        Op::Set => rest.to_owned(),
        Op::File => read_slot_file(name, rest, token)?,
    };
    if append {
        append_list_value(out, name, value);
    } else {
        out.insert(name.to_owned(), RawValue::Str(value));
    }
    Ok(())
}

/// Split a token at its FIRST operator character, `=` or `@`, whichever comes
/// first.
///
/// One rule for both operators, so neither needs quoting inside the other's
/// value: `summary=a@b` sets the literal `a@b`, and `summary@a=b` reads the
/// file named `a=b`. The same rule already decided that only the first `=`
/// separates; a second operator does not get a second rule.
/// One scan finds it, rather than a search per operator compared by index: a
/// comparison would be deriving "first" from two answers, and `at < eq` has
/// no `<=` case to distinguish (a byte is not both characters), so the
/// comparison is untestable where the scan is self-evident.
fn split_token(token: &str) -> Result<(&str, Op, &str), Diag> {
    let (index, op) = token
        .char_indices()
        .find_map(|(index, ch)| match ch {
            '=' => Some((index, Op::Set)),
            '@' => Some((index, Op::File)),
            _ => None,
        })
        .ok_or_else(|| {
            Diag::usage("not a key=value, key+=value or key@path assignment")
                .value(token)
                .hint(Hint::new("write it as --set name=value"))
        })?;
    let (key, rest) = token.split_at(index);
    Ok((key, op, rest.get(1..).unwrap_or_default()))
}

/// Read slot `name`'s value out of `path` — the `name@path` operator.
///
/// Trailing line terminators are stripped, and nothing else is. The operator
/// stands in for `--set "name=$(cat path)"`, and command substitution strips
/// them too, so a token swapped in for the substitution splices identically —
/// `contrib/ask-refactoring/size.sh` already depends on that, its spliced
/// bullets abutting the literal ones that follow the slot. Stripping is also
/// what keeps the operator usable for a typed slot at all: every editor ends
/// a text file with a newline, and `due@date.txt` would otherwise fail
/// coercion on a file whose visible contents are a perfectly good date.
///
/// Every other byte reaches the slot unchanged. What happens to it next is
/// the slot KIND's business, exactly as for a command-line value: a `text`
/// slot markup-escapes it, a `raw` slot splices it verbatim, and a typed slot
/// coerces it and fails naming itself. Nothing here converts anything —
/// notably, nothing turns Markdown into Typst, so a `.md` file fills a `text`
/// slot as the prose it literally is.
fn read_slot_file(name: &str, path: &str, token: &str) -> Result<String, Diag> {
    if path == "-" {
        return Err(Diag::usage("@- does not read stdin").value(token).hint(Hint::new(
            "pass a whole values mapping with -f -, where a YAML block scalar carries prose verbatim",
        )));
    }
    let text = std::fs::read_to_string(path).map_err(|err| {
        Diag::failed("can't read slot file")
            .value(path)
            .context("for slot", name)
            .io(&err)
    })?;
    Ok(text.trim_end_matches(['\r', '\n']).to_owned())
}

/// Append `item` to slot `name`'s list value (the `name+=item`/`+tag` shape),
/// creating the list on first use. A slot already set to a non-list value is
/// replaced outright — the last shape written wins.
pub fn append_list_value(values: &mut BTreeMap<String, RawValue>, name: &str, item: String) {
    match values
        .entry(name.to_owned())
        .or_insert_with(|| RawValue::List(Vec::new()))
    {
        RawValue::List(items) => items.push(item),
        slot => *slot = RawValue::List(vec![item]),
    }
}

#[cfg(test)]
#[path = "tokens_tests.rs"]
mod tests;
