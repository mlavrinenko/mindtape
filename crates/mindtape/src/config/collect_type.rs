//! The `returns` type expression of a collector.
//!
//! An atom (`bool`, `int`, `number`, `string`, `path`, `date`), a one-element
//! list `[T]`, or a record `{ field = T }`. The regex collector accepts only
//! `["string"]` or `["path"]`; the Typst collector accepts every shape.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Deserializer};

/// A `returns` type expression: an atom, a one-element list `[T]`, or a record
/// `{ field = T }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectorType {
    /// A scalar type.
    Atom(CollectorAtom),
    /// A list of one type, written `[T]`.
    List(Box<CollectorType>),
    /// A record of named fields, written `{ field = T }`.
    Record(BTreeMap<String, CollectorType>),
}

/// One atom of a [`CollectorType`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectorAtom {
    /// `true` or `false`.
    Bool,
    /// A whole number.
    Int,
    /// A whole or fractional number.
    Number,
    /// Text.
    String,
    /// A path, read as text but counted by `count_paths()`.
    Path,
    /// A `YYYY-MM-DD` date.
    Date,
}

impl CollectorAtom {
    /// Parse one atom name.
    ///
    /// # Errors
    /// A message naming the unknown atom and the known ones.
    pub fn parse(text: &str) -> Result<Self, String> {
        match text {
            "bool" => Ok(Self::Bool),
            "int" => Ok(Self::Int),
            "number" => Ok(Self::Number),
            "string" => Ok(Self::String),
            "path" => Ok(Self::Path),
            "date" => Ok(Self::Date),
            other => Err(format!(
                "unknown type: {}; use bool, int, number, string, path or date",
                crate::config::quote(other)
            )),
        }
    }

    /// The atom as a config spells it.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::Int => "int",
            Self::Number => "number",
            Self::String => "string",
            Self::Path => "path",
            Self::Date => "date",
        }
    }
}

impl CollectorType {
    /// The type as a config spells it, for `mt explain rules`.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::Atom(atom) => atom.as_str().to_owned(),
            Self::List(inner) => format!("[{}]", inner.describe()),
            Self::Record(fields) => {
                let fields: Vec<String> = fields
                    .iter()
                    .map(|(name, ty)| format!("{name} = {}", ty.describe()))
                    .collect();
                format!("{{ {} }}", fields.join(", "))
            }
        }
    }

    /// Whether this is a scalar, which a filter may compare and `--sort` may
    /// order.
    #[must_use]
    pub fn is_scalar(&self) -> bool {
        matches!(self, Self::Atom(_))
    }

    /// The type of one record field, or `None` for a non-record or an unknown
    /// field.
    #[must_use]
    pub fn field(&self, name: &str) -> Option<&Self> {
        match self {
            Self::Record(fields) => fields.get(name),
            _ => None,
        }
    }

    /// Whether this is the output of a regex query: `["string"]` or
    /// `["path"]`.
    #[must_use]
    pub fn is_regex_output(&self) -> bool {
        matches!(
            self,
            Self::List(inner)
                if matches!(
                    inner.as_ref(),
                    Self::Atom(CollectorAtom::String | CollectorAtom::Path)
                )
        )
    }

    /// Whether this is a list of paths, which `count_paths()` reads.
    #[must_use]
    pub fn is_path_list(&self) -> bool {
        matches!(
            self,
            Self::List(inner) if matches!(inner.as_ref(), Self::Atom(CollectorAtom::Path))
        )
    }

    /// Whether `value` matches this type.
    #[must_use]
    pub fn accepts(&self, value: &typst_harvest::HVal) -> bool {
        use typst_harvest::HVal;
        match self {
            Self::Atom(atom) => match (atom, value) {
                (CollectorAtom::Bool, HVal::Bool(_))
                | (CollectorAtom::Int, HVal::Int(_))
                | (CollectorAtom::Number, HVal::Int(_) | HVal::Float(_))
                | (CollectorAtom::String | CollectorAtom::Path, HVal::Str(_))
                | (CollectorAtom::Date, HVal::Date(_)) => true,
                (CollectorAtom::Date, HVal::Str(text)) => crate::date::parse_ymd(text).is_some(),
                _ => false,
            },
            Self::List(inner) => match value {
                HVal::Array(items) => items.iter().all(|item| inner.accepts(item)),
                _ => false,
            },
            Self::Record(fields) => match value {
                HVal::Dict(entries) => {
                    entries.len() == fields.len()
                        && fields.iter().all(|(name, ty)| {
                            entries.get(name).is_some_and(|value| ty.accepts(value))
                        })
                }
                _ => false,
            },
        }
    }
}

impl<'de> Deserialize<'de> for CollectorType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TypeVisitor;

        impl<'de> serde::de::Visitor<'de> for TypeVisitor {
            type Value = CollectorType;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a type name, a one-element array `[T]`, or a record `{ field = T }`")
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                CollectorAtom::parse(value)
                    .map(CollectorType::Atom)
                    .map_err(serde::de::Error::custom)
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<Self::Value, A::Error> {
                let inner: Option<CollectorType> = seq.next_element()?;
                let Some(inner) = inner else {
                    return Err(serde::de::Error::custom(
                        "a list type takes exactly one element: [T]",
                    ));
                };
                if seq.next_element::<CollectorType>()?.is_some() {
                    return Err(serde::de::Error::custom(
                        "a list type takes exactly one element: [T]",
                    ));
                }
                Ok(CollectorType::List(Box::new(inner)))
            }

            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                mut map: M,
            ) -> Result<Self::Value, M::Error> {
                let mut fields = BTreeMap::new();
                while let Some((name, ty)) = map.next_entry::<String, CollectorType>()? {
                    fields.insert(name, ty);
                }
                Ok(CollectorType::Record(fields))
            }
        }

        deserializer.deserialize_any(TypeVisitor)
    }
}

impl schemars::JsonSchema for CollectorType {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "CollectorType".into()
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        "mindtape::config::collect_type::CollectorType".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let inner = generator.subschema_for::<CollectorType>();
        schemars::json_schema!({
            "description": "A `returns` type expression: one of `bool`, `int`, \
                `number`, `string`, `path` and `date`; a one-element array `[T]` \
                for a list; or a table `{ field = T }` for a record.",
            "oneOf": [
                {
                    "type": "string",
                    "enum": ["bool", "int", "number", "string", "path", "date"]
                },
                {
                    "type": "array",
                    "minItems": 1,
                    "maxItems": 1,
                    "items": inner.clone(),
                    "description": "A list of one type, written `[T]`."
                },
                {
                    "type": "object",
                    "description": "A record of named fields, written `{ field = T }`.",
                    "additionalProperties": inner
                }
            ]
        })
    }
}

impl fmt::Display for CollectorType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.describe())
    }
}
