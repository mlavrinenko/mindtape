//! The lines under an error: [`Hint`] says what to do next, [`Note`] adds
//! context, and [`Command`] is a command line a hint tells the user to run.

use std::fmt;

use super::Diag;
use super::render::{quote, shell_word};

/// A `hint:` line. Its text is a `&'static str`; anything the user typed
/// reaches it only through a [`Command`] argument, a did-you-mean, or an
/// example rebuilt from the input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Hint(Shape);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Shape {
    Text(&'static str),
    Command(&'static str, Command),
    EndingIn(&'static str, Command),
    Run(Command, &'static str),
    DidYouMean(String),
    OneOf(Vec<String>),
    Typst(String),
    Example(&'static str),
    AsIn(&'static str, String),
    Project(String),
    NotInDict(mindtape::DictMiss),
}

impl Hint {
    /// A next step in words: `hint: quote a value: status == "wip"`.
    pub(crate) fn new(text: &'static str) -> Self {
        Self(Shape::Text(text))
    }

    /// A next step and the command that takes it:
    /// `hint: to add a note, pass it with -m: mt done a -m 'a note'`.
    pub(crate) fn command(text: &'static str, command: Command) -> Self {
        Self(Shape::Command(text, command))
    }

    /// A next step in words that ends in the command taking it:
    /// `hint: read them, then run mt allow`.
    pub(crate) fn ending_in(text: &'static str, command: Command) -> Self {
        Self(Shape::EndingIn(text, command))
    }

    /// A command and what it's for: `hint: run mt ls to see your tasks`.
    pub(crate) fn run(command: Command, purpose: &'static str) -> Self {
        Self(Shape::Run(command, purpose))
    }

    /// A correct use of what the user got wrong:
    /// `hint: for example: matches(title, "x")`.
    pub(crate) fn example(example: &'static str) -> Self {
        Self(Shape::Example(example))
    }

    /// A next step and the user's own input with it taken:
    /// `hint: to match text, quote it, as in status == "wip"`.
    pub(crate) fn as_in(text: &'static str, example: String) -> Self {
        Self(Shape::AsIn(text, example))
    }

    /// The known name nearest to what the user typed:
    /// `hint: did you mean alpha?`. A name, so it's bare.
    pub(crate) fn did_you_mean(name: impl Into<String>) -> Self {
        Self(Shape::DidYouMean(name.into()))
    }

    /// The names the user may pick from, as the library listed them:
    /// `hint: use one of: file, title`. Names are bare.
    pub(crate) fn one_of(names: Vec<String>) -> Self {
        Self(Shape::OneOf(names))
    }

    /// The word a dictionary lacks and the entries nearest it:
    /// `hint: 'Rote' is not in dict verbs; did you mean note, rate or rite?`.
    /// The word is the task's, so it's quoted; names are bare.
    pub(crate) fn not_in_dict(miss: mindtape::DictMiss) -> Self {
        Self(Shape::NotInDict(miss))
    }

    /// One of Typst's own hints on an error, in Typst's words. Typst's text
    /// is not ours to fix as a `&'static str`, so only [`Diag::typst_hints`]
    /// builds one.
    ///
    /// [`Diag::typst_hints`]: super::Diag::typst_hints
    pub(super) fn typst(text: String) -> Self {
        Self(Shape::Typst(text))
    }

    /// The hint as the set member `name` gives it: the command it names runs
    /// in that member.
    fn in_member(self, name: &str) -> Self {
        Self(match self.0 {
            Shape::Command(text, command) => Shape::Command(text, command.in_member(name)),
            Shape::EndingIn(text, command) => Shape::EndingIn(text, command.in_member(name)),
            Shape::Run(command, purpose) => Shape::Run(command.in_member(name), purpose),
            shape @ (Shape::Text(_)
            | Shape::DidYouMean(_)
            | Shape::OneOf(_)
            | Shape::Typst(_)
            | Shape::Example(_)
            | Shape::AsIn(..)
            | Shape::Project(_)
            | Shape::NotInDict(_)) => shape,
        })
    }
}

impl fmt::Display for Hint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Shape::Text(text) => formatter.write_str(text),
            Shape::Command(text, command) => write!(formatter, "{text}: {command}"),
            Shape::EndingIn(text, command) => write!(formatter, "{text} {command}"),
            Shape::Run(command, purpose) => write!(formatter, "run {command} {purpose}"),
            Shape::DidYouMean(name) => write!(formatter, "did you mean {name}?"),
            Shape::Typst(text) | Shape::Project(text) => formatter.write_str(text),
            Shape::OneOf(names) => write!(formatter, "use one of: {}", names.join(", ")),
            Shape::Example(example) => write!(formatter, "for example: {example}"),
            Shape::AsIn(text, example) => write!(formatter, "{text}, as in {example}"),
            Shape::NotInDict(miss) => not_in_dict(formatter, miss),
        }
    }
}

/// `'<word>' is not in dict <dict>`, then `; did you mean a, b or c?` when an
/// entry is near.
fn not_in_dict(formatter: &mut fmt::Formatter<'_>, miss: &mindtape::DictMiss) -> fmt::Result {
    write!(
        formatter,
        "{} is not in dict {}",
        quote(&miss.word),
        miss.dict
    )?;
    match miss.nearest.as_slice() {
        [] => Ok(()),
        [only] => write!(formatter, "; did you mean {only}?"),
        [rest @ .., last] => write!(formatter, "; did you mean {} or {last}?", rest.join(", ")),
    }
}

/// The one CLI caller of `suggest::nearest`: clippy refuses every other.
#[allow(clippy::disallowed_methods, reason = "the one sanctioned caller")]
fn nearest_hint(name: &str, known: &[&str], show: impl FnOnce(&str) -> String) -> Option<Hint> {
    mindtape::suggest::nearest(name, known).map(|near| Hint::did_you_mean(show(near)))
}

impl Diag {
    /// The known name nearest to `name` as a did-you-mean hint, or the
    /// diagnostic unchanged when none is close.
    pub(crate) fn did_you_mean(self, name: &str, known: &[&str]) -> Self {
        self.did_you_mean_with(name, known, str::to_owned)
    }

    /// [`Self::did_you_mean`], or `fallback` when no name is close.
    pub(crate) fn did_you_mean_or(self, name: &str, known: &[&str], fallback: Hint) -> Self {
        self.did_you_mean_or_with(name, known, str::to_owned, fallback)
    }

    /// [`Self::did_you_mean`], with the nearest name worded by `show`, such
    /// as `@alias` for a worktree alias.
    pub(crate) fn did_you_mean_with(
        self,
        name: &str,
        known: &[&str],
        show: impl FnOnce(&str) -> String,
    ) -> Self {
        match nearest_hint(name, known, show) {
            Some(hint) => self.hint(hint),
            None => self,
        }
    }

    /// [`Self::did_you_mean_with`], or `fallback` when no name is close.
    pub(crate) fn did_you_mean_or_with(
        self,
        name: &str,
        known: &[&str],
        show: impl FnOnce(&str) -> String,
        fallback: Hint,
    ) -> Self {
        self.hint(nearest_hint(name, known, show).unwrap_or(fallback))
    }

    /// `hint` as the only hint, in place of the ones the message came with,
    /// when the caller knows better what the user should do.
    pub(crate) fn only_hint(mut self, hint: Hint) -> Self {
        self.0.hints = vec![hint];
        self
    }

    /// The project's own advice, worded in its config (a gate's `message`),
    /// as a hint. Counts toward the two-hint limit.
    ///
    /// # Panics
    /// In a debug build, on a third hint.
    pub(crate) fn project_hint(self, text: impl Into<String>) -> Self {
        self.hint(Hint(Shape::Project(text.into())))
    }

    /// The problem as the set member `name` reports it: every command its
    /// hints name runs in that member, and a note names it.
    ///
    /// ```text
    /// hint: run mt @blog flip --list to see the statuses
    /// note: project: blog
    /// ```
    pub(crate) fn in_member(mut self, name: &str) -> Self {
        self.0.hints = std::mem::take(&mut self.0.hints)
            .into_iter()
            .map(|hint| hint.in_member(name))
            .collect();
        self.note(Note::new("project").name(name))
    }
}

/// A `note:` line: context the reader may need, with no action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Note {
    text: &'static str,
    /// What follows the text, already quoted when it's a value.
    tail: Option<String>,
    /// A continuation line of an unconverted multi-line message.
    legacy: Option<String>,
}

impl Note {
    /// A note in words.
    pub(crate) fn new(text: &'static str) -> Self {
        Self {
            text,
            tail: None,
            legacy: None,
        }
    }

    /// The value the note is about, rendered single-quoted after the text.
    pub(crate) fn value(mut self, value: impl AsRef<str>) -> Self {
        self.tail = Some(quote(value.as_ref()));
        self
    }

    /// The name the note is about, a stem or a field, rendered bare.
    pub(crate) fn name(mut self, name: impl Into<String>) -> Self {
        self.tail = Some(name.into());
        self
    }

    /// A line already worded elsewhere, such as another task's breach, kept as
    /// it is.
    pub(crate) fn line(text: impl Into<String>) -> Self {
        Self {
            legacy: Some(text.into()),
            ..Self::new("")
        }
    }

    /// A line of an unconverted message past its first, kept word for word.
    pub(super) fn legacy(line: &str) -> Self {
        Self {
            legacy: Some(line.to_owned()),
            ..Self::new("")
        }
    }
}

impl fmt::Display for Note {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(line) = &self.legacy {
            return formatter.write_str(line);
        }
        formatter.write_str(self.text)?;
        match &self.tail {
            Some(tail) => write!(formatter, ": {tail}"),
            None => Ok(()),
        }
    }
}

/// A command line in a hint: literal words, and arguments the user typed,
/// which are quoted only when a shell needs them to be, so the line pastes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Command {
    /// The words after `mt` that pick the project it runs on: this run's own
    /// `@name` or `-w <dir>`, or a set member's `@name`. `None` when the run
    /// found its project from where it stood, and for a global command.
    scope: Option<String>,
    /// The words after the scope: a subcommand, its flags and arguments.
    words: Vec<String>,
    /// Whether the command reads no project, as `mt install` doesn't, so no
    /// scope is put on it.
    global: bool,
}

impl Command {
    /// `mt`, for a command that runs on a project, to which words and
    /// arguments are added. Under `mt @blog` or `mt -w <dir>`, it carries the
    /// same words, so it runs on the same project as printed.
    pub(crate) fn mt() -> Self {
        Self::under(crate::selector::hint())
    }

    /// The core of [`Self::mt`]: `mt` on the project `scope` picks.
    pub(super) fn under(scope: Option<String>) -> Self {
        Self {
            scope,
            words: Vec::new(),
            global: false,
        }
    }

    /// `mt`, for a command that reads no project, such as `mt install`.
    pub(crate) fn mt_global() -> Self {
        Self {
            global: true,
            ..Self::under(None)
        }
    }

    /// A literal word: a subcommand or a flag.
    pub(crate) fn word(mut self, word: &'static str) -> Self {
        self.words.push(word.to_owned());
        self
    }

    /// An argument the user typed or a name from the project, quoted for the
    /// shell when it has to be.
    pub(crate) fn arg(mut self, arg: impl AsRef<str>) -> Self {
        self.words.push(shell_word(arg.as_ref()));
        self
    }

    /// The command run in the set member `name`, as `mt @<name> …`, in place
    /// of the run's own scope. A global command stays as it is.
    fn in_member(mut self, name: &str) -> Self {
        if !self.global {
            self.scope = Some(shell_word(&format!("@{name}")));
        }
        self
    }
}

impl fmt::Display for Command {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("mt")?;
        for word in self.scope.iter().chain(&self.words) {
            write!(formatter, " {word}")?;
        }
        Ok(())
    }
}
