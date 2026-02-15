//! Task ID utilities: `UUIDv7` generation, base62 encoding/decoding.
//!
//! IDs are always stored in canonical hyphenated `UUIDv7` form. Base62 is an
//! alternative compact encoding accepted on input and used for display.

use uuid::Uuid;

/// Generate a new `UUIDv7` and return its hyphenated string.
#[must_use]
pub fn new_id() -> String {
    Uuid::now_v7().to_string()
}

/// Encode a canonical `UUIDv7` string as base62.
///
/// # Errors
///
/// Returns `None` if the input is not a valid UUID.
#[must_use]
pub fn encode_base62(uuid_str: &str) -> Option<String> {
    let uuid = Uuid::parse_str(uuid_str).ok()?;
    Some(base62::encode(uuid.as_u128()))
}

/// Parse a task ID that may be either a hyphenated `UUIDv7` or a base62-encoded
/// `UUIDv7`. Returns the canonical hyphenated form.
///
/// # Errors
///
/// Returns an error message if the input is neither a valid `UUIDv7` nor a
/// valid base62 encoding of one.
pub fn parse_task_id(input: &str) -> Result<String, String> {
    // Try parsing as a standard UUID first.
    if let Ok(uuid) = Uuid::parse_str(input) {
        return if uuid.get_version() == Some(uuid::Version::SortRand) {
            Ok(uuid.to_string())
        } else {
            Err(format!("'{input}' is a valid UUID but not UUIDv7"))
        };
    }

    // Try decoding as base62.
    match base62::decode(input.as_bytes()) {
        Ok(value) => {
            let uuid = Uuid::from_u128(value);
            if uuid.get_version() == Some(uuid::Version::SortRand) {
                Ok(uuid.to_string())
            } else {
                Err(format!("'{input}' decodes to a UUID but not UUIDv7"))
            }
        }
        Err(_) => Err(format!("'{input}' is not a valid UUIDv7 or base62 ID")),
    }
}

/// Check whether a string is a valid task ID (`UUIDv7` or base62-encoded `UUIDv7`).
#[must_use]
pub fn is_valid_task_id(input: &str) -> bool {
    parse_task_id(input).is_ok()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    const SAMPLE_UUID: &str = "019c5b9b-7317-77b1-bf52-ce7a298cfcad";

    #[test]
    fn new_id_is_valid_uuidv7() {
        let id = new_id();
        let uuid = Uuid::parse_str(&id).unwrap();
        assert_eq!(uuid.get_version(), Some(uuid::Version::SortRand));
    }

    #[test]
    fn encode_base62_roundtrip() {
        let b62 = encode_base62(SAMPLE_UUID).unwrap();
        assert!(!b62.is_empty());
        assert!(b62.len() < SAMPLE_UUID.len()); // base62 is shorter

        // Decode back
        let parsed = parse_task_id(&b62).unwrap();
        assert_eq!(parsed, SAMPLE_UUID);
    }

    #[test]
    fn parse_accepts_uuid() {
        let result = parse_task_id(SAMPLE_UUID).unwrap();
        assert_eq!(result, SAMPLE_UUID);
    }

    #[test]
    fn parse_accepts_base62() {
        let b62 = encode_base62(SAMPLE_UUID).unwrap();
        let result = parse_task_id(&b62).unwrap();
        assert_eq!(result, SAMPLE_UUID);
    }

    #[test]
    fn parse_rejects_uuidv4() {
        let v4 = Uuid::new_v4().to_string();
        let result = parse_task_id(&v4);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not UUIDv7"));
    }

    #[test]
    fn parse_rejects_garbage() {
        let result = parse_task_id("not-a-uuid-at-all");
        assert!(result.is_err());
    }

    #[test]
    fn encode_rejects_invalid_uuid() {
        assert!(encode_base62("not-a-uuid").is_none());
    }

    #[test]
    fn is_valid_accepts_both_formats() {
        let b62 = encode_base62(SAMPLE_UUID).unwrap();
        assert!(is_valid_task_id(SAMPLE_UUID));
        assert!(is_valid_task_id(&b62));
    }

    #[test]
    fn is_valid_rejects_invalid() {
        assert!(!is_valid_task_id("garbage"));
        assert!(!is_valid_task_id(&Uuid::new_v4().to_string()));
    }

    #[test]
    fn new_id_roundtrips_through_base62() {
        let id = new_id();
        let b62 = encode_base62(&id).unwrap();
        let back = parse_task_id(&b62).unwrap();
        assert_eq!(back, id);
    }
}
