//! Tests for `[[actors]]` / `[[roles]]` parsing and cross-checks.
#![allow(clippy::unwrap_used)]

use crate::config::Config;
use crate::parse_config;

use super::{Actor, ActorKind, Role, roles_of};

#[test]
fn absent_sections_are_empty_and_valid() {
    let config = parse_config("[tasks]\nroots = [\"tasks\"]\n").unwrap();
    assert!(config.actors.is_empty());
    assert!(config.roles.is_empty());
    assert!(config.validate().is_ok());
}

#[test]
fn actors_and_roles_parse() {
    let config = parse_config(
        "[[roles]]\nname = \"reviewer\"\n\n\
         [[actors]]\nname = \"ada\"\nkind = \"human\"\nroles = [\"reviewer\"]\n",
    )
    .unwrap();
    assert_eq!(
        config.roles,
        vec![Role {
            name: "reviewer".to_owned()
        }]
    );
    assert_eq!(
        config.actors,
        vec![Actor {
            name: "ada".to_owned(),
            kind: ActorKind::Human,
            roles: vec!["reviewer".to_owned()],
        }]
    );
}

#[test]
fn actor_without_roles_parses() {
    let config = parse_config("[[actors]]\nname = \"ada\"\nkind = \"human\"\n").unwrap();
    let actor = config.actors.first().unwrap();
    assert!(actor.roles.is_empty());
}

#[test]
fn kind_outside_the_closed_set_is_rejected_at_parse() {
    let err = parse_config("[[actors]]\nname = \"ada\"\nkind = \"bot\"\n").unwrap_err();
    assert!(
        err.to_string().contains("expected `human` or `llm`"),
        "{}",
        err.to_string()
    );
}

#[test]
fn kind_is_required() {
    let err = parse_config("[[actors]]\nname = \"ada\"\n").unwrap_err();
    assert!(err.to_string().contains("kind"), "{}", err.to_string());
}

#[test]
fn duplicate_role_names_are_rejected() {
    let config =
        parse_config("[[roles]]\nname = \"reviewer\"\n\n[[roles]]\nname = \"reviewer\"\n").unwrap();
    let err = config.validate().unwrap_err();
    assert!(err.contains("duplicate role"), "{err}");
}

#[test]
fn empty_role_name_is_rejected() {
    let config = parse_config("[[roles]]\nname = \"\"\n").unwrap();
    let err = config.validate().unwrap_err();
    assert!(err.contains("must not be empty"), "{err}");
}

#[test]
fn duplicate_actor_names_are_rejected() {
    let config = parse_config(
        "[[actors]]\nname = \"ada\"\nkind = \"human\"\n\n\
         [[actors]]\nname = \"ada\"\nkind = \"llm\"\n",
    )
    .unwrap();
    let err = config.validate().unwrap_err();
    assert!(err.contains("duplicate actor"), "{err}");
}

#[test]
fn actor_referencing_undeclared_role_is_rejected() {
    let config =
        parse_config("[[actors]]\nname = \"ada\"\nkind = \"human\"\nroles = [\"reviewer\"]\n")
            .unwrap();
    let err = config.validate().unwrap_err();
    assert!(err.contains("`reviewer` is not a declared role"), "{err}");
}

#[test]
fn assignee_allowed_must_name_a_declared_actor() {
    let config = parse_config(
        "[tasks.assignee]\nallowed = [\"eve\"]\n\n\
         [[actors]]\nname = \"ada\"\nkind = \"human\"\n",
    )
    .unwrap();
    let err = config.validate().unwrap_err();
    assert!(err.contains("`eve` is not a declared actor"), "{err}");
}

#[test]
fn assignee_allowed_referencing_a_declared_actor_passes() {
    let config = parse_config(
        "[tasks.assignee]\nallowed = [\"ada\"]\n\n\
         [[actors]]\nname = \"ada\"\nkind = \"human\"\n",
    )
    .unwrap();
    assert!(config.validate().is_ok());
}

#[test]
fn unknown_actor_key_is_rejected() {
    let err = parse_config("[[actors]]\nname = \"ada\"\nkind = \"human\"\nprompt = \"hi\"\n")
        .unwrap_err();
    assert!(err.to_string().contains("prompt"), "{}", err.to_string());
}

#[test]
fn roles_of_resolves_from_the_registry() {
    let config = parse_config(
        "[[roles]]\nname = \"reviewer\"\n\n\
         [[actors]]\nname = \"ada\"\nkind = \"human\"\nroles = [\"reviewer\"]\n",
    )
    .unwrap();
    assert_eq!(
        roles_of("ada", &config),
        Some(["reviewer".to_owned()].as_slice())
    );
    assert_eq!(roles_of("eve", &config), None);
    assert_eq!(roles_of("ada", &Config::default()), None);
}
