#![allow(clippy::unwrap_used)]

use std::path::Path;

use super::{Chain, MAX_DEPTH};
use crate::diag::{assert_plain, plain};

/// The chain a command sees after the flips `links` started, each a
/// `(path, id, status)`.
fn chain_of(links: &[(&str, &str, &str)]) -> Chain {
    links
        .iter()
        .fold(Chain::default(), |chain, (path, id, status)| {
            Chain::parse(
                &chain
                    .passed_down(Path::new(path), id, status)
                    .to_string_lossy(),
            )
        })
}

#[test]
fn a_passed_down_chain_reads_back_whole() {
    let chain = chain_of(&[("/p/a.typ", "a", "done"), ("/p/b.typ", "b", "wip")]);

    assert_eq!(chain.labels(), "a → done, b → wip");
}

#[test]
fn an_empty_chain_admits_any_flip() {
    assert!(
        Chain::default()
            .admit(Path::new("/p/a.typ"), "a", "done")
            .is_ok()
    );
}

#[test]
fn a_flip_in_progress_is_refused_with_the_chain() {
    let chain = chain_of(&[("/p/a.typ", "a", "done"), ("/p/b.typ", "b", "done")]);

    let refusal = chain.admit(Path::new("/p/a.typ"), "a", "done").unwrap_err();

    assert_plain(
        &refusal,
        "error: flip already in progress: 'a → done'\n\
         hint: a hook can't flip the task it runs for into the same status again\n\
         note: in progress: a → done, b → done\n",
    );
}

/// The pair is the task and the status: another status, or another project's
/// task with the same id, is a different flip.
#[test]
fn the_same_task_into_another_status_or_another_file_is_admitted() {
    let chain = chain_of(&[("/p/a.typ", "a", "done")]);

    assert!(chain.admit(Path::new("/p/a.typ"), "a", "wip").is_ok());
    assert!(chain.admit(Path::new("/q/a.typ"), "a", "done").is_ok());
}

#[test]
fn a_chain_max_depth_deep_refuses_the_next_flip() {
    let paths: Vec<String> = (0..MAX_DEPTH).map(|n| format!("/p/t{n}.typ")).collect();
    let links: Vec<(&str, &str, &str)> = paths.iter().map(|p| (p.as_str(), "t", "done")).collect();
    let deep = chain_of(&links);
    let shallower = chain_of(links.get(1..).unwrap());

    let refusal = deep
        .admit(Path::new("/p/new.typ"), "new", "done")
        .unwrap_err();

    let text = plain(&refusal);
    assert!(
        text.starts_with("error: flips nested too deep: 'new → done'"),
        "{text}"
    );
    assert!(
        shallower
            .admit(Path::new("/p/new.typ"), "new", "done")
            .is_ok()
    );
}

#[test]
fn a_line_without_three_fields_is_skipped() {
    let chain = Chain::parse("done\ta\t/p/a.typ\ngarbage\n");

    assert_eq!(chain.labels(), "a → done");
}
