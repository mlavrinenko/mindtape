//! A question to the person at the terminal. It is the one thing `mt` writes
//! to stderr that is neither a [`Diag`] nor a command's report.
//!
//! A prompt is not a diagnostic: nothing failed, so it carries no `error:`
//! prefix and no hint, and it reads as the plain text a person answers.
//! Stdout stays for the report, so a pipe such as `mt rm | less` doesn't
//! swallow the question. The command lists what it will do, one line each, and
//! [`confirm`] writes them, asks `Continue? [y/N] ` and reads the answer.
//!
//! The command checks for a terminal before it builds the lines, because a
//! prompt nobody can answer is a refusal with a hint, which is a [`Diag`].

use std::io::{self, BufRead, Write};

use super::Diag;

/// The question that ends every prompt.
const QUESTION: &str = "Continue? [y/N] ";

/// Write `lines` and the question to stderr, then read the answer from stdin:
/// `true` for `y` or `yes` in any case, `false` for anything else, an empty
/// line included.
pub(crate) fn confirm(lines: &[String]) -> Result<bool, Diag> {
    ask(&mut io::stderr().lock(), &mut io::stdin().lock(), lines)
}

/// [`confirm`] with the streams given, so a test can answer it.
fn ask(out: &mut impl Write, input: &mut impl BufRead, lines: &[String]) -> Result<bool, Diag> {
    write_prompt(out, lines).map_err(|err| Diag::failed("can't write the prompt").io(&err))?;
    let mut answer = String::new();
    input
        .read_line(&mut answer)
        .map_err(|err| Diag::failed("can't read the answer").io(&err))?;
    let answer = answer.trim().to_ascii_lowercase();
    Ok(answer == "y" || answer == "yes")
}

fn write_prompt(out: &mut impl Write, lines: &[String]) -> io::Result<()> {
    for line in lines {
        writeln!(out, "{line}")?;
    }
    write!(out, "{QUESTION}")?;
    out.flush()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn run(answer: &str) -> (bool, String) {
        let mut out = Vec::new();
        let lines = vec![
            "rm will delete 1 task(s):".to_owned(),
            "  - a.typ".to_owned(),
        ];
        let got = ask(&mut out, &mut answer.as_bytes(), &lines).unwrap();
        (got, String::from_utf8(out).unwrap())
    }

    #[test]
    fn the_lines_come_first_and_the_question_last_with_no_prefix() {
        let (_, written) = run("n\n");
        assert_eq!(
            written,
            "rm will delete 1 task(s):\n  - a.typ\nContinue? [y/N] "
        );
    }

    #[test]
    fn y_and_yes_in_any_case_confirm() {
        for answer in ["y\n", "yes\n", "Y\n", " YES \n"] {
            assert!(run(answer).0, "{answer:?}");
        }
    }

    #[test]
    fn anything_else_declines() {
        for answer in ["n\n", "\n", "", "yep\n", "no\n"] {
            assert!(!run(answer).0, "{answer:?}");
        }
    }

    /// A stream that fails every call, to reach the two error arms of `ask`.
    struct Broken;

    impl Write for Broken {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("pipe closed"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl io::Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("terminal gone"))
        }
    }

    #[test]
    fn a_closed_stderr_fails_the_prompt_instead_of_declining() {
        let err = ask(&mut Broken, &mut "y\n".as_bytes(), &[]).unwrap_err();
        assert_eq!(
            super::super::render::render(&err, false),
            "error: can't write the prompt: pipe closed\n"
        );
    }

    #[test]
    fn an_unreadable_stdin_fails_the_prompt_instead_of_declining() {
        let mut input = io::BufReader::new(Broken);
        let err = ask(&mut Vec::new(), &mut input, &[]).unwrap_err();
        assert_eq!(
            super::super::render::render(&err, false),
            "error: can't read the answer: terminal gone\n"
        );
    }
}
