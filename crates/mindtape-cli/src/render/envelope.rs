//! The `$schema` envelope every object output carries.
//!
//! [`Envelope`] names the schema the object conforms to and flattens the
//! payload beside it, so a consumer reads the output's own keys at the top
//! level. `$schema` is written first, and the payload follows in declaration
//! order.

use serde::Serialize;

use crate::cmd::schema::SchemaId;

/// A machine output wrapped in the URL of its schema.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct Envelope<T> {
    /// The URL of the schema this object conforms to.
    #[serde(rename = "$schema")]
    pub schema: String,
    /// The output itself, flattened into the envelope.
    #[serde(flatten)]
    pub payload: T,
}

impl<T> Envelope<T> {
    /// Wrap `payload` in the schema `id` names.
    #[must_use]
    pub fn new(id: SchemaId, payload: T) -> Self {
        Self {
            schema: id.url(),
            payload,
        }
    }
}

#[cfg(test)]
#[path = "envelope_tests.rs"]
mod tests;
