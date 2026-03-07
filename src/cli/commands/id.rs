use anyhow::Result;
use clap::{Parser, ValueEnum};

use mindtape_store::id;

/// Generate, validate, or convert a task ID.
///
/// With no arguments, generates a new base62-encoded `UUIDv7`.
/// With `--from`, converts or validates the given ID.
/// Use `--to` to choose the output format (default: base62).
#[derive(Parser, Debug)]
pub struct IdArgs {
    /// Input ID to validate or convert (`UUIDv7` or base62).
    /// When omitted, a new ID is generated.
    #[arg(long)]
    pub from: Option<String>,

    /// Output format
    #[arg(long, value_enum, default_value_t = IdFormat::Base62)]
    pub to: IdFormat,
}

/// Output format for task IDs.
#[derive(Clone, Debug, ValueEnum)]
pub enum IdFormat {
    /// Base62-encoded compact form
    Base62,
    /// Hyphenated `UUIDv7`
    #[value(name = "uuidv7")]
    UuidV7,
}

impl IdArgs {
    /// Run the id command.
    ///
    /// # Errors
    /// Returns error if a given ID fails validation or conversion.
    pub fn run(&self) -> Result<()> {
        let canonical = match &self.from {
            Some(input) => id::parse_task_id(input).map_err(|e| anyhow::anyhow!("{e}"))?,
            None => id::new_id(),
        };

        match self.to {
            IdFormat::Base62 => {
                let b62 = id::encode_base62(&canonical)
                    .ok_or_else(|| anyhow::anyhow!("failed to encode as base62"))?;
                print!("{b62}");
            }
            IdFormat::UuidV7 => {
                print!("{canonical}");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    const SAMPLE_UUID: &str = "019c5b9b-7317-77b1-bf52-ce7a298cfcad";

    // --- Generation ---

    #[test]
    fn generate_base62() {
        let args = IdArgs {
            from: None,
            to: IdFormat::Base62,
        };
        assert!(args.run().is_ok());
    }

    #[test]
    fn generate_uuidv7() {
        let args = IdArgs {
            from: None,
            to: IdFormat::UuidV7,
        };
        assert!(args.run().is_ok());
    }

    // --- Validation (same-format round-trip) ---

    #[test]
    fn validate_uuid_to_uuidv7() {
        let args = IdArgs {
            from: Some(SAMPLE_UUID.to_string()),
            to: IdFormat::UuidV7,
        };
        assert!(args.run().is_ok());
    }

    #[test]
    fn validate_base62_to_base62() {
        let b62 = id::encode_base62(SAMPLE_UUID).unwrap();
        let args = IdArgs {
            from: Some(b62),
            to: IdFormat::Base62,
        };
        assert!(args.run().is_ok());
    }

    // --- Conversion ---

    #[test]
    fn convert_uuid_to_base62() {
        let expected = id::encode_base62(SAMPLE_UUID).unwrap();
        let args = IdArgs {
            from: Some(SAMPLE_UUID.to_string()),
            to: IdFormat::Base62,
        };
        assert!(args.run().is_ok());
        assert_eq!(id::encode_base62(SAMPLE_UUID).unwrap(), expected);
    }

    #[test]
    fn convert_base62_to_uuidv7() {
        let b62 = id::encode_base62(SAMPLE_UUID).unwrap();
        let args = IdArgs {
            from: Some(b62),
            to: IdFormat::UuidV7,
        };
        assert!(args.run().is_ok());
    }

    // --- Rejection ---

    #[test]
    fn rejects_invalid_input() {
        let args = IdArgs {
            from: Some("not-valid".to_string()),
            to: IdFormat::Base62,
        };
        assert!(args.run().is_err());
    }

    #[test]
    fn rejects_uuidv4() {
        let v4 = uuid::Uuid::new_v4().to_string();
        let args = IdArgs {
            from: Some(v4),
            to: IdFormat::Base62,
        };
        assert!(args.run().is_err());
    }
}
