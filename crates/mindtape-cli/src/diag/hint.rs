//! The lines under an error: [`Hint`] says what to do next, [`Note`] adds
//! context, and [`Command`] is a command line a hint tells the user to run.

use std::fmt;

use super::Diag;
use super::render::{quote, shell_word};

/// A `hint:` line. Its text is a `&'static str`; anything the user typed
/// reaches it only through a [`Command`] argument or a did-you-mean.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Hint(Shape);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Shape {
    Text(&'static str),
    Command(&'static str, Command),
    Run(Command, &'static str),
    DidYouMean(String),
    Typst(String),
    Example(&'static str),
    Project(String),
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

    /// A command and what it's for: `hint: run mt ls to see your tasks`.
    pub(crate) fn run(command: Command, purpose: &'static str) -> Self {
        Self(Shape::Run(command, purpose))
    }

    /// A correct use of what the user got wrong:
    /// `hint: for example: matches(title, "x")`.
    pub(crate) fn example(example: &'static str) -> Self {
        Self(Shape::Example(example))
    }

    /// The known name nearest to what the user typed:
    /// `hint: did you mean alpha?`. A name, so it's bare.
    pub(crate) fn did_you_mean(name: impl Into<String>) -> Self {
        Self(Shape::DidYouMean(name.into()))
    }

    /// One of Typst's own hints on an error, in Typst's words. Typst's text
    /// is not ours to fix as a `&'static str`, so only [`Diag::typst_hints`]
    /// builds one.
    ///
    /// [`Diag::typst_hints`]: super::Diag::typst_hints
    pub(super) fn typst(text: String) -> Self {
        Self(Shape::Typst(text))
    }
}

impl fmt::Display for Hint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Shape::Text(text) => formatter.write_str(text),
            Shape::Command(text, command) => write!(formatter, "{text}: {command}"),
            Shape::Run(command, purpose) => write!(formatter, "run {command} {purpose}"),
            Shape::DidYouMean(name) => write!(formatter, "did you mean {name}?"),
            Shape::Typst(text) | Shape::Project(text) => formatter.write_str(text),
            Shape::Example(example) => write!(formatter, "for example: {example}"),
        }
    }
}

impl Diag {
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
pub(crate) struct Command(Vec<String>);

impl Command {
    /// `mt`, to which words and arguments are added.
    pub(crate) fn mt() -> Self {
        Self(vec!["mt".to_owned()])
    }

    /// A literal word: a subcommand or a flag.
    pub(crate) fn word(mut self, word: &'static str) -> Self {
        self.0.push(word.to_owned());
        self
    }

    /// An argument the user typed or a name from the project, quoted for the
    /// shell when it has to be.
    pub(crate) fn arg(mut self, arg: impl AsRef<str>) -> Self {
        self.0.push(shell_word(arg.as_ref()));
        self
    }
}

impl fmt::Display for Command {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0.join(" "))
    }
}
