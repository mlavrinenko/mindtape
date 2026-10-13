use super::Problem;
use crate::suggest::Unknown;

fn with_hint(text: &str) -> Problem {
    Problem::new(text).with(Unknown {
        nearest: Some("backend".to_owned()),
        names: vec!["backend".to_owned()],
    })
}

/// What a user reads is the message alone: the suggestion travels as data and
/// the CLI words it as `hint:` lines.
#[test]
fn displays_the_message_without_the_suggestion() {
    assert_eq!(
        with_hint("tags: unknown tag").to_string(),
        "tags: unknown tag"
    );
}

#[test]
fn compares_to_text_by_message_alone() {
    let problem = with_hint("tags: unknown tag");
    assert!(problem == *"tags: unknown tag");
    assert!(problem == "tags: unknown tag");
    assert!(problem != "tags: other");
}
