use anyhow::{Result, bail};
use clap::Parser;

use mindtape_store::id;

/// Generate or validate a task ID.
///
/// With no arguments, generates a new `UUIDv7` in base62 format.
/// With `--raw`, outputs the plain hyphenated `UUIDv7` instead.
/// With a positional `<ID>`, validates that it is a valid task ID.
#[derive(Parser, Debug)]
pub struct IdArgs {
    /// ID to validate (`UUIDv7` or base62)
    pub id: Option<String>,

    /// Output plain hyphenated `UUIDv7` instead of base62
    #[arg(long)]
    pub raw: bool,
}

impl IdArgs {
    /// Run the id command.
    ///
    /// # Errors
    /// Returns error if a given ID fails validation.
    pub fn run(&self) -> Result<()> {
        if let Some(input) = &self.id {
            // Validate mode
            match id::parse_task_id(input) {
                Ok(_) => Ok(()),
                Err(err) => bail!("{err}"),
            }
        } else {
            // Generate mode
            let uuid = id::new_id();
            if self.raw {
                print!("{uuid}");
            } else {
                // Safety: encode_base62 only returns None for invalid UUIDs;
                // new_id() always produces a valid one.
                let b62 = id::encode_base62(&uuid).unwrap_or(uuid);
                print!("{b62}");
            }
            Ok(())
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn generate_produces_valid_id() {
        let args = IdArgs { id: None, raw: false };
        assert!(args.run().is_ok());
    }

    #[test]
    fn generate_raw_produces_valid_id() {
        let args = IdArgs { id: None, raw: true };
        assert!(args.run().is_ok());
    }

    #[test]
    fn validate_accepts_uuid() {
        let args = IdArgs {
            id: Some("019c5b9b-7317-77b1-bf52-ce7a298cfcad".to_string()),
            raw: false,
        };
        assert!(args.run().is_ok());
    }

    #[test]
    fn validate_accepts_base62() {
        let b62 = id::encode_base62("019c5b9b-7317-77b1-bf52-ce7a298cfcad").unwrap();
        let args = IdArgs {
            id: Some(b62),
            raw: false,
        };
        assert!(args.run().is_ok());
    }

    #[test]
    fn validate_rejects_invalid() {
        let args = IdArgs {
            id: Some("not-valid".to_string()),
            raw: false,
        };
        assert!(args.run().is_err());
    }

    #[test]
    fn validate_rejects_uuidv4() {
        let v4 = uuid::Uuid::new_v4().to_string();
        let args = IdArgs {
            id: Some(v4),
            raw: false,
        };
        assert!(args.run().is_err());
    }
}
