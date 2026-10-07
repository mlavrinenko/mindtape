//! Where a config key sits in a config file, so a load error can say
//! `.mindtape/config.toml:12:` instead of only naming the key.

use std::path::{Path, PathBuf};

use toml_edit::{ImDocument, Item, Table, TableLike, Value};

use crate::config::layers::{Layer, LayerSource};

/// `value` in single quotes, the way the CLI's error format quotes a value.
pub(crate) fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}

/// A line of a config file, as the user should read it: the path is relative
/// to the project root when the file sits under it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigPlace {
    /// The config file.
    pub file: PathBuf,
    /// The 1-based line of the key.
    pub line: usize,
}

/// The place of the key a problem message starts with (`views.x.sort: ...`),
/// searched from the highest-precedence layer down. `None` when the message
/// leads with no key, or no layer file declares it.
pub(crate) fn place_of(message: &str, layers: &[Layer], root: &Path) -> Option<ConfigPlace> {
    let key = message.lines().next()?.split_once(": ")?.0;
    if !key.chars().all(is_key_char) {
        return None;
    }
    layers.iter().rev().find_map(|layer| {
        let file = match &layer.source {
            LayerSource::Embedded => return None,
            LayerSource::Xdg(path) | LayerSource::Dir(path) | LayerSource::Project(path) => path,
        };
        let line = key_line(&layer.text, key)?;
        let shown = file.strip_prefix(root).unwrap_or(file);
        Some(ConfigPlace {
            file: shown.to_path_buf(),
            line,
        })
    })
}

fn is_key_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '[' | ']')
}

/// The 1-based line of `path` (`on.flip[0].actions[1]`) in `text`, or of the
/// deepest ancestor the text declares.
fn key_line(text: &str, path: &str) -> Option<usize> {
    let doc = ImDocument::parse(text).ok()?;
    let mut node = Node::Item(doc.as_item());
    let mut start = None;
    for step in steps(path) {
        let Some(next) = descend(node, &step) else {
            break;
        };
        start = next.start.or(start);
        node = next.node;
    }
    Some(text[..start?].matches('\n').count() + 1)
}

/// A step down, and the byte offset of the key or entry it landed on.
struct Landed<'a> {
    node: Node<'a>,
    start: Option<usize>,
}

fn descend<'a>(node: Node<'a>, step: &Step) -> Option<Landed<'a>> {
    match step {
        Step::Key(name) => {
            let (key, child) = node.table_like()?.get_key_value(name)?;
            Some(Landed {
                node: Node::Item(child),
                start: key.span().map(|span| span.start),
            })
        }
        Step::Index(index) => {
            let element = node.element(*index)?;
            Some(Landed {
                node: element,
                start: element.span().map(|span| span.start),
            })
        }
    }
}

/// Somewhere in the parsed document: an item, an array-of-tables entry, or an
/// element of an inline array.
#[derive(Clone, Copy)]
enum Node<'a> {
    Item(&'a Item),
    Table(&'a Table),
    Value(&'a Value),
}

impl<'a> Node<'a> {
    fn table_like(self) -> Option<&'a dyn TableLike> {
        match self {
            Self::Item(item) => item.as_table_like(),
            Self::Table(table) => Some(table),
            Self::Value(value) => value.as_inline_table().map(|t| t as &dyn TableLike),
        }
    }

    fn element(self, index: usize) -> Option<Self> {
        match self {
            Self::Item(Item::ArrayOfTables(tables)) => tables.get(index).map(Self::Table),
            Self::Item(Item::Value(value)) | Self::Value(value) => {
                value.as_array()?.get(index).map(Self::Value)
            }
            Self::Item(_) | Self::Table(_) => None,
        }
    }

    fn span(self) -> Option<std::ops::Range<usize>> {
        match self {
            Self::Item(item) => item.span(),
            Self::Table(table) => table.span(),
            Self::Value(value) => value.span(),
        }
    }
}

enum Step {
    Key(String),
    Index(usize),
}

/// `a.b[1].c` as `Key(a) Key(b) Index(1) Key(c)`.
fn steps(path: &str) -> Vec<Step> {
    let mut out = Vec::new();
    for part in path.split('.') {
        let (name, rest) = part.split_once('[').unwrap_or((part, ""));
        if !name.is_empty() {
            out.push(Step::Key(name.to_owned()));
        }
        for index in rest.split('[') {
            if let Some(number) = index.strip_suffix(']').and_then(|n| n.parse().ok()) {
                out.push(Step::Index(number));
            }
        }
    }
    out
}

#[cfg(test)]
#[path = "locate_tests.rs"]
mod tests;
