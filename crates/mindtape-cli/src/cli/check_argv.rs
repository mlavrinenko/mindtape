//! `mt _cli --check-argv` — judge one command line the way `mt` would parse it.
//!
//! The doc gate `just check-doc-spans` finds `mt ...` command lines in the
//! docs and hands each to this check, so a doc span is accepted or refused by
//! the parser that runs it. The line goes through the same steps as a real
//! run: [`cmd::worktrees::desugar_argv`]'s `@name` rewrite, then clap, then the
//! status-verb parser for a shortcut such as `mt done`. A line that only lacks
//! a required argument passes, because a doc span shows part of a command.
//!
//! A word the doc writes as a stand-in (`<cmd>`, `TERM`, `...`) is never
//! refused: a subcommand or argument clap rejects as unknown is accepted when
//! it is a stand-in.

use std::ffi::OsString;
use std::path::PathBuf;

use crate::diag::Result;
use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{CommandFactory, Parser};

use crate::cli::Cli;
use crate::{cmd::flip, cmd::worktrees};

/// Judge `line` with `statuses` as the declared status verbs.
///
/// # Errors
/// Returns the refusal message when `mt` would not parse the line.
pub fn run(line: &str, statuses: &[String]) -> Result<()> {
    check(line, statuses)
        .map_err(|msg| crate::diag::Diag::verbatim(crate::diag::Kind::Refused, msg))
}

fn check(line: &str, statuses: &[String]) -> Result<(), String> {
    let words = split_words(line);
    if words.first().map(String::as_str) != Some("mt") {
        return Err(format!("not an mt command line: {line}"));
    }
    // `@name` needs no real worktree here: the sugar only has to desugar.
    let argv = worktrees::desugar_with(words.iter().map(OsString::from).collect(), |_| {
        Ok(PathBuf::from("."))
    })
    .map_err(|err| err.to_string())?;
    match Cli::try_parse_from(argv) {
        Ok(cli) => cli
            .status_shortcut()
            .map_or(Ok(()), |external| check_status(external, statuses)),
        Err(err) => judge(&err),
    }
}

/// `mt <verb> ...` where clap saw no command of that name: a declared status
/// verb parses through the shortcut parser, anything else is no command.
fn check_status(external: &[String], statuses: &[String]) -> Result<(), String> {
    let verb = external.first().map_or("", String::as_str);
    if statuses.iter().any(|s| s == verb) {
        return flip::shortcut_command(verb)
            .try_get_matches_from(external)
            .map(|_| ())
            .or_else(|err| judge(&err));
    }
    if is_placeholder(verb) {
        return Ok(());
    }
    let cmd = Cli::command();
    let mut known: Vec<&str> = cmd
        .get_subcommands()
        .filter(|c| c.get_name() != "help" && !c.is_hide_set())
        .map(clap::Command::get_name)
        .collect();
    known.extend(statuses.iter().map(String::as_str));
    Err(format!(
        "no subcommand {verb}{}",
        closest(mindtape::suggest::nearest(verb, &known))
    ))
}

/// Turn a clap refusal into the message the gate reports, or accept it when
/// it only says the line is partial.
fn judge(err: &clap::Error) -> Result<(), String> {
    match err.kind() {
        ErrorKind::DisplayHelp
        | ErrorKind::DisplayVersion
        | ErrorKind::MissingRequiredArgument
        | ErrorKind::MissingSubcommand
        | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => Ok(()),
        // `--sort` alone: the option is real, the span just stops before its value.
        ErrorKind::InvalidValue if context(err, ContextKind::InvalidValue).concat().is_empty() => {
            Ok(())
        }
        ErrorKind::ValueValidation | ErrorKind::InvalidValue
            if is_placeholder(&context(err, ContextKind::InvalidValue).concat()) =>
        {
            Ok(())
        }
        ErrorKind::UnknownArgument => {
            let arg = context(err, ContextKind::InvalidArg).concat();
            if is_placeholder(&arg) {
                return Ok(());
            }
            let path = command_path(err);
            let hint = closest(context(err, ContextKind::SuggestedArg).first().cloned());
            Err(if arg.starts_with('-') {
                format!("{path} has no flag {arg}{hint}")
            } else {
                format!("{path} takes no argument {arg}")
            })
        }
        ErrorKind::InvalidSubcommand => {
            let name = context(err, ContextKind::InvalidSubcommand).concat();
            if is_placeholder(&name) {
                return Ok(());
            }
            let hint = closest(
                context(err, ContextKind::SuggestedSubcommand)
                    .first()
                    .cloned(),
            );
            Err(format!(
                "{} has no subcommand {name}{hint}",
                command_path(err)
            ))
        }
        _ => Err(first_line(err)),
    }
}

fn closest<S: AsRef<str>>(name: Option<S>) -> String {
    name.map(|n| format!("; closest: {}", n.as_ref().trim_matches('\'')))
        .unwrap_or_default()
}

fn first_line(err: &clap::Error) -> String {
    let text = err.to_string();
    let line = text.lines().next().unwrap_or_default();
    line.strip_prefix("error: ").unwrap_or(line).to_owned()
}

/// The strings clap attached to `kind` on `err`.
fn context(err: &clap::Error, kind: ContextKind) -> Vec<String> {
    match err.get(kind) {
        Some(ContextValue::String(s)) => vec![s.clone()],
        Some(ContextValue::Strings(v)) => v.clone(),
        Some(ContextValue::StyledStr(s)) => vec![s.to_string()],
        _ => Vec::new(),
    }
}

/// The command the error is about (`mt ls`), read from clap's usage line.
fn command_path(err: &clap::Error) -> String {
    let usage = context(err, ContextKind::Usage).concat();
    let words: Vec<&str> = usage
        .trim_start_matches("Usage:")
        .split_whitespace()
        .take_while(|w| !w.starts_with(['[', '<', '-']))
        .collect();
    if words.is_empty() {
        "mt".to_owned()
    } else {
        words.join(" ")
    }
}

/// A word the docs write as a stand-in: `<task>`, `TERM`, `...`, `*`.
fn is_placeholder(word: &str) -> bool {
    word.contains(['<', '>', '…', '*'])
        || word.contains("...")
        || (!word.is_empty()
            && word.starts_with(|c: char| c.is_ascii_uppercase())
            && word.chars().all(|c| c.is_ascii_uppercase() || c == '_'))
}

/// Split on whitespace, keeping a quoted stretch whole and dropping its quotes.
fn split_words(line: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut cur = String::new();
    let mut open = false;
    let mut quote = None;
    for c in line.chars() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (None, '\'' | '"') => {
                quote = Some(c);
                open = true;
            }
            (None, c) if c.is_whitespace() => {
                if open || !cur.is_empty() {
                    words.push(std::mem::take(&mut cur));
                    open = false;
                }
            }
            (_, c) => cur.push(c),
        }
    }
    if open || !cur.is_empty() {
        words.push(cur);
    }
    words
}

#[cfg(test)]
#[path = "check_argv_tests.rs"]
mod tests;
