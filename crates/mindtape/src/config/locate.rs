//! Where a config error comes from: the file, and the line of the key in it.
//! [`Sources`] holds the text of every file a merged config was built from,
//! the layers and each layer's packs, in merge precedence. A load error asks
//! it for `.mindtape/config.toml:12:` ([`Sources::place_of`]) or for the file
//! that holds a key serde refuses ([`Sources::blame`]), so a key a pack or an
//! ancestor `defaults.toml` sets is never charged to the project config.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use toml_edit::{ImDocument, Item, Table, TableLike, Value};

use crate::config::layers::{Layer, LayerSource};
use crate::config::origins::{KeySlot, ParamOrigin};
use crate::config::pack_registry::{PACK_FILE, PackOrigin, RuleOrigin};
use crate::config::packs_toml::header_lines;
use crate::config::{Config, ConfigError};

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

/// One file a merged config was built from.
#[derive(Debug)]
struct Source {
    origin: RuleOrigin,
    /// How an error names it: relative to the project root when under it, or
    /// `pack <name>` for a pack compiled into the binary.
    file: PathBuf,
    text: String,
}

/// A merged rule array's entry, so the index an error leads with, which
/// counts the merged array, can be turned into the entry's index in the one
/// file that wrote it.
#[derive(Debug)]
struct RuleSlot {
    /// The array's dotted path, such as `tasks.rules`.
    array: String,
    /// The entry's index in the merged array.
    index: usize,
    /// Its file, as an index into [`Sources::items`]. `None` for an entry of
    /// the embedded floor, which is no file.
    source: Option<usize>,
    /// Its index among that file's own `[[array]]` tables, `None` when the
    /// file writes the array inline.
    local: Option<usize>,
}

/// Every file a merged config was built from, lowest precedence first.
#[derive(Debug, Default)]
pub(crate) struct Sources {
    items: Vec<Source>,
    rules: Vec<RuleSlot>,
    /// Each leaf key's winning source, by dotted path.
    pub(super) keys: BTreeMap<String, KeySlot>,
    /// Each pack parameter, resolved.
    pub(super) params: Vec<ParamOrigin>,
}

/// [`Sources`] as a loaded [`Config`] keeps it: shared by every clone and never
/// equal-compared, as two configs of one declared content are one config
/// wherever its keys came from.
#[derive(Debug, Clone, Default)]
pub(crate) struct Provenance(Arc<Sources>);

impl Provenance {
    pub(crate) fn new(sources: Sources) -> Self {
        Self(Arc::new(sources))
    }
}

impl std::ops::Deref for Provenance {
    type Target = Sources;

    fn deref(&self) -> &Sources {
        &self.0
    }
}

impl PartialEq for Provenance {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Sources {
    /// The layers alone, for an error raised before any pack resolved.
    pub(crate) fn of_layers(layers: &[Layer], root: &Path) -> Self {
        let mut sources = Self::default();
        for layer in layers {
            sources.push_layer(layer, root);
        }
        sources
    }

    /// Add a layer's file. The embedded floor is no file, so it is skipped.
    pub(crate) fn push_layer(&mut self, layer: &Layer, root: &Path) {
        let (LayerSource::Xdg(path) | LayerSource::Dir(path) | LayerSource::Project(path)) =
            &layer.source
        else {
            return;
        };
        self.items.push(Source {
            origin: RuleOrigin::Layer(layer.source.clone()),
            file: shown(path, root),
            text: layer.text.clone(),
        });
    }

    /// Add each pack's file, in `extends` order.
    pub(crate) fn push_packs(&mut self, packs: &[(PackOrigin, String)], root: &Path) {
        for (origin, text) in packs {
            self.push_pack(origin, text, root);
        }
    }

    /// Add a pack's file: its path, its `pack.toml` for a directory pack, or
    /// `pack <source>:<pack>` for one compiled in or cached from a source.
    pub(crate) fn push_pack(&mut self, origin: &PackOrigin, text: &str, root: &Path) {
        let file = match origin {
            PackOrigin::Embedded(name) | PackOrigin::Cached { name, .. } => {
                PathBuf::from(format!("pack {name}"))
            }
            PackOrigin::Path(path) => shown(path, root),
            PackOrigin::Dir(dir) => shown(&dir.join(PACK_FILE), root),
        };
        self.items.push(Source {
            origin: RuleOrigin::Pack(origin.clone()),
            file,
            text: text.to_owned(),
        });
    }

    /// Record where each entry of the merged rule array at `array` came from,
    /// each as its origin and the line of its `[[array]]` header.
    pub(crate) fn add_rule_slots<'a>(
        &mut self,
        array: &[&str],
        entries: impl Iterator<Item = (&'a RuleOrigin, Option<usize>)>,
    ) {
        for (index, (origin, line)) in entries.enumerate() {
            let source = self.items.iter().position(|item| item.origin == *origin);
            let local = source.zip(line).and_then(|(source, line)| {
                let headers = header_lines(&self.items.get(source)?.text, array);
                headers.iter().position(|header| *header == line)
            });
            self.rules.push(RuleSlot {
                array: array.join("."),
                index,
                source,
                local,
            });
        }
    }

    /// The place of the key a problem message starts with (`views.x.sort:
    /// ...`), in the highest-precedence file that declares the whole key.
    /// `None` when the message leads with no key, or no file declares it.
    pub(crate) fn place_of(&self, message: &str) -> Option<ConfigPlace> {
        let key = message.lines().next()?.split_once(": ")?.0;
        if !key.chars().all(is_key_char) {
            return None;
        }
        if let Some((slot, rest)) = self.rule_slot(key) {
            let item = self.items.get(slot.source?)?;
            let local = format!("{}[{}]{rest}", slot.array, slot.local?);
            return place_in(item, &local);
        }
        self.items.iter().rev().find_map(|item| place_in(item, key))
    }

    /// The merged rule entry a key such as `tasks.rules[2].assert` is about,
    /// and the rest of the key after the index.
    fn rule_slot<'k>(&self, key: &'k str) -> Option<(&RuleSlot, &'k str)> {
        self.rules.iter().find_map(|slot| {
            let rest = key.strip_prefix(&slot.array)?.strip_prefix('[')?;
            let (index, rest) = rest.split_once(']')?;
            (index.parse() == Ok(slot.index)).then_some((slot, rest))
        })
    }

    /// The error for a merged document serde refused: when a file, read
    /// alone, fails with the same message, that file is the one at fault, and
    /// its error carries the line. The highest-precedence such file wins. A
    /// failure that only the merge causes has no file, and stays as it was.
    pub(crate) fn blame(&self, merged: toml::de::Error) -> ConfigError {
        let culprit = self.items.iter().rev().find_map(|item| {
            let alone = toml::from_str::<Config>(&item.text).err()?;
            (alone.message() == merged.message()).then(|| ConfigError::ParseIn {
                origin: item.file.display().to_string(),
                error: alone,
            })
        });
        culprit.unwrap_or(ConfigError::Parse(merged))
    }
}

/// `path` relative to `root` when it sits under it.
fn shown(path: &Path, root: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

/// The line of the whole `key` in `item`, if it declares it.
fn place_in(item: &Source, key: &str) -> Option<ConfigPlace> {
    Some(ConfigPlace {
        file: item.file.clone(),
        line: key_line(&item.text, key)?,
    })
}

fn is_key_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '[' | ']')
}

/// The 1-based line of `path` (`on.flip[0].actions[1]`) in `text`, or `None`
/// when the text doesn't declare every step of it. A deepest ancestor would
/// point at a key that doesn't hold the value.
fn key_line(text: &str, path: &str) -> Option<usize> {
    let doc = ImDocument::parse(text).ok()?;
    let mut node = Node::Item(doc.as_item());
    let mut start = None;
    for step in steps(path) {
        let next = descend(node, &step)?;
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
