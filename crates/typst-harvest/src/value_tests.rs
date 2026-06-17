#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use typst::foundations::{Datetime, NativeElement, Value};
use typst_library::model::{LinkElem, Url};
use typst_library::text::TextElem;

use super::{HVal, convert, format_date};

#[test]
fn projects_standard_link_to_its_destination() {
    // `link("stem.typ")` is the standard Typst hyperlink element; harvest reads
    // its URL destination, not the rendered element's repr.
    let url = Url::new("converge-speconaut.typ").unwrap();
    let link = Value::Content(LinkElem::from_url(url).pack());
    assert_eq!(
        convert(&link),
        HVal::Str("converge-speconaut.typ".to_owned())
    );
}

#[test]
fn projects_content_to_its_plain_text() {
    // A trailing `[note]` block reaches harvest as content; project it to the
    // concatenated text, not the element's repr.
    let note = Value::Content(TextElem::new("shim gone".into()).pack());
    assert_eq!(convert(&note), HVal::Str("shim gone".to_owned()));
}

#[test]
fn converts_scalars() {
    assert_eq!(convert(&Value::None), HVal::Null);
    assert_eq!(convert(&Value::Bool(true)), HVal::Bool(true));
    assert_eq!(convert(&Value::Int(7)), HVal::Int(7));
    assert_eq!(convert(&Value::Float(1.5)), HVal::Float(1.5));
    assert_eq!(
        convert(&Value::Str("hi".into())),
        HVal::Str("hi".to_owned())
    );
}

#[test]
fn converts_date() {
    let date = Datetime::from_ymd(2026, 6, 14).unwrap();
    assert_eq!(format_date(&date), "2026-06-14");
    assert_eq!(
        convert(&Value::Datetime(date)),
        HVal::Date("2026-06-14".to_owned())
    );
}

#[test]
fn converts_nested() {
    let array = Value::Array(typst::foundations::Array::from_iter([
        Value::Int(1),
        Value::Str("a".into()),
    ]));
    let HVal::Array(items) = convert(&array) else {
        panic!("expected array");
    };
    assert_eq!(items.len(), 2);
    assert_eq!(items.first(), Some(&HVal::Int(1)));
}

#[test]
fn numeric_accessors_accept_int_and_float() {
    assert_eq!(HVal::Int(3).as_i64(), Some(3));
    assert_eq!(HVal::Int(3).as_f64(), Some(3.0));
    assert_eq!(HVal::Float(2.5).as_f64(), Some(2.5));
    assert_eq!(HVal::Float(2.5).as_i64(), None);
}

#[test]
fn dict_lookup() {
    let mut map = BTreeMap::new();
    map.insert("marker".to_owned(), HVal::Str("mindtape.task".to_owned()));
    let dict = HVal::Dict(map);
    assert_eq!(
        dict.get("marker").and_then(HVal::as_str),
        Some("mindtape.task")
    );
    assert_eq!(dict.get("missing"), None);
}

#[test]
fn null_and_string_helpers() {
    assert!(HVal::Null.is_null());
    assert!(!HVal::Bool(false).is_null());
    assert_eq!(HVal::Str("x".to_owned()).as_str(), Some("x"));
    assert_eq!(
        HVal::Date("2026-01-01".to_owned()).as_str(),
        Some("2026-01-01")
    );
}
