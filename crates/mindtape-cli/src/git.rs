//! Repository plumbing for `cleanup`, hosted in the consumer-blind
//! [`typst_cleanup`] crate (gix-backed: locate the work-tree root, guard against
//! a dirty working tree). Re-exported here so the command and its tests keep
//! referring to `crate::git`.

pub use typst_cleanup::{is_clean, toplevel};
