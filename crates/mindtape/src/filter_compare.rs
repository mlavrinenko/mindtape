//! The comparison arm of the filter walk, split out of `filter.rs`
//! (file-size gate): resolving each side — a field, a `changed()` or
//! `count()` count, a literal or `now()` — and ordering the two under the
//! missing-field rule.

use evalexpr::{DefaultNumericTypes, Node, Operator};

use crate::config::Config;
use crate::expr::{
    call_args, compare_ordering, extract_identifier, resolve_now_or_literal, restore_name,
    unwrap_root,
};
use crate::fields::FieldValue;
use crate::filter::{Ctx, FilterError, eval_bool, field_value};
use crate::filter_predicates::changed_count;
use crate::model::Task;
use crate::numeric::format_num;

/// A comparison. The left side is a field or a `changed()`/`count()` count;
/// the right side a literal, `now()`, another field, or a count.
pub(crate) fn comparison(node: &Node<DefaultNumericTypes>, ctx: &Ctx) -> Result<bool, FilterError> {
    let [lhs, rhs] = node.children() else {
        return Err(FilterError::Invalid(
            "comparison needs two operands".to_owned(),
        ));
    };
    let op = node.operator();
    if let Some(right) = operand(rhs, ctx)? {
        return Ok(compare_values(op, &left_operand(lhs, ctx)?, &right));
    }
    let literal = resolve_now_or_literal(rhs).map_err(FilterError::Invalid)?;
    let value = if extract_identifier(lhs).is_ok_and(|field| field == "role") {
        role_field(ctx.task, ctx.config, &literal)
    } else {
        left_operand(lhs, ctx)?
    };
    Ok(compare(op, &value, &literal))
}

/// A comparison side that resolves on the task rather than being written out:
/// a field, a `changed()` count, or a `count()` count. `None` for a literal or
/// `now()`.
fn operand(node: &Node<DefaultNumericTypes>, ctx: &Ctx) -> Result<Option<FieldValue>, FilterError> {
    let node = unwrap_root(node);
    match node.operator() {
        Operator::VariableIdentifierRead { identifier } => {
            field_value(ctx.task, &restore_name(identifier), ctx.config).map(Some)
        }
        Operator::FunctionIdentifier { identifier } if identifier == "changed" => {
            changed_count(node.children(), ctx.config, ctx.dry)
                .map(|count| Some(FieldValue::Num(count)))
        }
        Operator::FunctionIdentifier { identifier } if identifier == "count" => {
            count(node.children(), ctx).map(|total| Some(FieldValue::Num(total)))
        }
        _ => Ok(None),
    }
}

/// `count(<expr>)`: how many loaded tasks satisfy `<expr>`, each read as the
/// task in context — see [`crate::filter_aggregate`].
fn count(children: &[Node<DefaultNumericTypes>], ctx: &Ctx) -> Result<f64, FilterError> {
    let [arg] = call_args(children)[..] else {
        return Err(FilterError::Invalid(
            "count() needs one argument".to_owned(),
        ));
    };
    let total = ctx
        .readiness
        .set()
        .count(arg, ctx.task, |task| eval_bool(arg, &ctx.over(task)))?;
    // A task count is far below 2^52, so the conversion is exact.
    #[allow(clippy::cast_precision_loss)]
    let total = total as f64;
    Ok(total)
}

/// The left side of a comparison, which must be a field or a count — also
/// what a task-scope commit rule's `staged(<field>)` resolves to
/// ([`crate::commit_task_expr`]).
pub(crate) fn left_operand(
    node: &Node<DefaultNumericTypes>,
    ctx: &Ctx,
) -> Result<FieldValue, FilterError> {
    match operand(node, ctx)? {
        Some(value) => Ok(value),
        None => Err(FilterError::Invalid(
            extract_identifier(node).err().unwrap_or_default(),
        )),
    }
}

/// `role` is a virtual, comparison-only field resolved from the actors
/// registry at read time: a task matches a role literal when its assignee is a
/// declared actor holding that role. Every other comparison resolves to
/// [`FieldValue::Missing`], so the missing-field rule applies uniformly and
/// `role == "reviewer"` / `role != "reviewer"` partition the whole set with no
/// third case.
fn role_field(task: &Task, config: &Config, literal: &str) -> FieldValue {
    match task
        .assignee
        .as_deref()
        .and_then(|name| crate::config_actors::roles_of(name, config))
    {
        Some(roles) if roles.iter().any(|role| role == literal) => {
            FieldValue::Str(literal.to_owned())
        }
        _ => FieldValue::Missing,
    }
}

/// Compare one resolved field value against a literal.
///
/// Missing-field rule: a field the task does not carry is unequal to every
/// literal and orders against none — `!=` holds, `==` / `<` / `>` / `<=` / `>=`
/// are all false. So `kind != "umbrella"` matches a task with no `kind` at all,
/// and the pair `kind == "umbrella"` / `kind != "umbrella"` partitions the whole
/// set with no third case. `[[tasks.limits]]`
/// ([`crate::config_limits`]) rests on that: one limit over each half of such a
/// pair caps every task by construction, including the ones leaving the field
/// unset. Pinned by `missing_field_is_unequal_to_every_literal` in
/// `filter_tests.rs` — do not relax it without revisiting that rule.
pub(crate) fn compare(
    op: &Operator<DefaultNumericTypes>,
    value: &FieldValue,
    literal: &str,
) -> bool {
    match value {
        FieldValue::Missing => matches!(op, Operator::Neq),
        FieldValue::Str(text) => compare_ordering(op, Some(text.as_str().cmp(literal))),
        FieldValue::Num(num) => match literal.parse::<f64>() {
            Ok(rhs) => compare_ordering(op, num.partial_cmp(&rhs)),
            Err(_) => compare_ordering(op, Some(format_num(*num).as_str().cmp(literal))),
        },
    }
}

/// Compare two resolved values. A missing side follows the missing-field rule
/// of [`compare`]; two numbers compare numerically; otherwise the right side's
/// text stands in as the literal.
pub(crate) fn compare_values(
    op: &Operator<DefaultNumericTypes>,
    left: &FieldValue,
    right: &FieldValue,
) -> bool {
    match (left, right) {
        (FieldValue::Num(lhs), FieldValue::Num(rhs)) => compare_ordering(op, lhs.partial_cmp(rhs)),
        (_, FieldValue::Missing) | (FieldValue::Missing, _) => matches!(op, Operator::Neq),
        (_, FieldValue::Str(text)) => compare(op, left, text),
        (_, FieldValue::Num(num)) => compare(op, left, &format_num(*num)),
    }
}
