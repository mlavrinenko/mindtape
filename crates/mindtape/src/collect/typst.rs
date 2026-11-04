//! The Typst-function collector: call a project function over one task and
//! publish its value as `derived.<name>`.
//!
//! The call reuses [`crate::rules::typst::CompiledCheck`] — the same probe,
//! world and shared [`typst_harvest::SourceSnapshot`] a `[[tasks.rules]]
//! check` uses. The function receives the whole canonical task, or the field
//! `query.field` names, exactly as a check does. A missing export is refused
//! at config load ([`crate::rules::typst::validate_collector_load`]); an eval
//! error, a `panic()` or a value that does not match `returns` is one
//! [`Problem`] naming the module and function, which [`crate::validate`]
//! reports as a `Collector` violation at `derived.<name>`.

use std::path::Path;

use typst_harvest::HVal;

use crate::config::Config;
use crate::config::collect::{CollectorSpec, CollectorType, TypstQuery};
use crate::model::Task;
use crate::problem::Problem;
use crate::rules::CheckSpec;
use crate::rules::typst::CompiledCheck;

/// One Typst collector, compiled once for a run.
pub(crate) struct CompiledTypst {
    check: CompiledCheck,
    returns: CollectorType,
    name: String,
    module: String,
    function: String,
}

impl CompiledTypst {
    /// Build the reusable check state for `spec`'s Typst query.
    pub(crate) fn new(
        spec: &CollectorSpec,
        query: &TypstQuery,
        root: &Path,
        config: &Config,
    ) -> Self {
        let check = CheckSpec {
            typst: query.typst.clone(),
            function: query.function.clone(),
            field: query.field.clone(),
            root: spec.root.clone(),
        };
        Self {
            check: CompiledCheck::new(&check, root, config),
            returns: spec.returns.clone(),
            name: spec.name.clone(),
            module: query.typst.clone(),
            function: query.function.clone(),
        }
    }

    /// The value the function returned, or one problem naming the module and
    /// function. A wrong-typed value is a problem too.
    pub(crate) fn evaluate(&self, task: &Task, config: &Config) -> Result<HVal, Problem> {
        let fail = |message: String| {
            Problem::new(format!(
                "derived.{}: {} in {}: {message}",
                self.name, self.function, self.module
            ))
        };
        let value = self.check.evaluate_value(task, config).map_err(fail)?;
        if !self.returns.accepts(&value) {
            return Err(fail(format!(
                "returned {}; expected {}",
                value.to_typst(),
                self.returns.describe()
            )));
        }
        Ok(value)
    }
}
