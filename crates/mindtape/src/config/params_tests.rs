#![allow(clippy::unwrap_used)]

use super::{CallError, rewrite_calls};
use crate::filter::Filter;

/// `expr` with every `param()` call naming `p` replaced by `7`.
fn rewrite(expr: &str) -> Result<String, CallError> {
    rewrite_calls(expr, |name| (name == "p").then(|| "7".to_owned()))
}

#[test]
fn a_call_with_spaces_around_its_argument_is_rewritten() {
    assert_eq!(rewrite("x <= param ( \"p\" ) + 1").unwrap(), "x <= 7 + 1");
}

#[test]
fn a_longer_name_holding_param_is_no_call() {
    for expr in [
        "my-param(\"p\")",
        "x.param(\"p\")",
        "params(\"p\")",
        "_param(\"p\")",
    ] {
        assert_eq!(rewrite(expr).unwrap(), expr);
    }
}

#[test]
fn a_call_after_a_number_and_a_minus_is_a_call() {
    assert_eq!(rewrite("5-param(\"p\")").unwrap(), "5-7");
}

#[test]
fn a_bare_param_with_no_parenthesis_is_a_field() {
    assert_eq!(rewrite("param == 3").unwrap(), "param == 3");
}

#[test]
fn an_escaped_quote_inside_a_string_keeps_the_string_open() {
    let expr = r#"title == "a \" param(\"p\")" && param("p")"#;
    assert_eq!(
        rewrite(expr).unwrap(),
        r#"title == "a \" param(\"p\")" && 7"#
    );
}

#[test]
fn a_call_that_is_not_one_quoted_name_is_malformed() {
    for expr in ["param(p)", "param(\"p\"", "param(\"p\", 2)", "param(\"p"] {
        assert_eq!(rewrite(expr), Err(CallError::Malformed), "{expr}");
    }
}

#[test]
fn an_unknown_name_is_reported_with_its_escapes_read() {
    assert_eq!(
        rewrite(r#"param("q\"")"#),
        Err(CallError::Unknown("q\"".to_owned()))
    );
}

#[test]
fn a_string_literal_parses_back_as_the_same_string() {
    let literal = super::literal(&toml::Value::String(r#"a "b" \ c"#.to_owned()));
    let filter = Filter::parse(&format!("title == {literal}")).unwrap();
    let task = crate::model::Task {
        title: r#"a "b" \ c"#.to_owned(),
        ..crate::model::Task::default()
    };
    assert!(
        filter
            .matches(&task, &crate::config::Config::default())
            .unwrap()
    );
}
