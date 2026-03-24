use std::cmp::Ordering;

use super::macros::*;
use super::{Arg, Args, FfiContext, Result};
use crate::array::ArrayValue;
use crate::interpreter::RuntimeErrorKind;
use crate::quantity::{Quantity, QuantityError, QuantityOrdering};
use crate::typechecker::type_scheme::TypeScheme;
use crate::typed_ast::Type;
use crate::value::Value;

fn array_element_type(type_: &TypeScheme) -> TypeScheme {
    let Type::Array(element_type) = type_.to_concrete_type() else {
        unreachable!("array builtins must receive arrays");
    };
    TypeScheme::concrete(*element_type)
}

fn array_return_element_type(type_: &TypeScheme) -> TypeScheme {
    let Type::Array(element_type) = type_.to_concrete_type() else {
        unreachable!("array builtins returning arrays must declare array return types");
    };
    TypeScheme::concrete(*element_type)
}

fn callable_return_type(type_: &TypeScheme) -> TypeScheme {
    let Type::Fn(_, return_type) = type_.to_concrete_type() else {
        unreachable!("higher-order array builtins must receive callable arguments");
    };
    TypeScheme::concrete(*return_type)
}

#[derive(Clone)]
struct SortEntry {
    value: Value,
    key: Quantity,
}

fn merge_sorted(
    mut lhs: Vec<SortEntry>,
    mut rhs: Vec<SortEntry>,
) -> Result<Vec<SortEntry>, Box<RuntimeErrorKind>> {
    let mut merged = Vec::with_capacity(lhs.len() + rhs.len());
    lhs.reverse();
    rhs.reverse();

    while let (Some(lhs_head), Some(rhs_head)) = (lhs.last(), rhs.last()) {
        let take_lhs = match lhs_head.key.partial_cmp_preserve_nan(&rhs_head.key) {
            QuantityOrdering::IncompatibleUnits => {
                return Err(Box::new(RuntimeErrorKind::QuantityError(
                    QuantityError::IncompatibleUnits(
                        lhs_head.key.unit().clone(),
                        rhs_head.key.unit().clone(),
                    ),
                )));
            }
            QuantityOrdering::NanOperand => false,
            QuantityOrdering::Ok(Ordering::Less) => true,
            QuantityOrdering::Ok(Ordering::Equal | Ordering::Greater) => false,
        };

        if take_lhs {
            merged.push(lhs.pop().unwrap());
        } else {
            merged.push(rhs.pop().unwrap());
        }
    }

    while let Some(entry) = lhs.pop() {
        merged.push(entry);
    }
    while let Some(entry) = rhs.pop() {
        merged.push(entry);
    }

    Ok(merged)
}

fn merge_sort_entries(entries: Vec<SortEntry>) -> Result<Vec<SortEntry>, Box<RuntimeErrorKind>> {
    if entries.len() <= 1 {
        return Ok(entries);
    }

    let midpoint = entries.len() / 2;
    let rhs = entries[midpoint..].to_vec();
    let lhs = entries[..midpoint].to_vec();

    merge_sorted(merge_sort_entries(lhs)?, merge_sort_entries(rhs)?)
}

pub fn len(
    _ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    let array = arg!(args).unsafe_as_array();

    return_scalar!(array.len_axis0() as f64)
}

pub fn range(
    _ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    let start = scalar_arg!(args).to_f64();
    let end = scalar_arg!(args).to_f64();

    if !start.is_finite() || !end.is_finite() || start.fract() != 0.0 || end.fract() != 0.0 {
        return Err(Box::new(RuntimeErrorKind::EmptyArray));
    }

    let start = start as i64;
    let end = end as i64;

    let values = if start <= end {
        (start..=end)
            .map(|n| Value::Quantity(Quantity::from_scalar(n as f64)))
            .collect()
    } else {
        Vec::new()
    };

    Ok(Value::Array(ArrayValue::from_values(values)))
}

pub fn fill(
    _ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    let value = arg!(args);
    let shape = arg!(args).unsafe_as_array();
    Ok(ArrayValue::filled(shape, value)?.into_value())
}

pub fn zeros(
    _ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    Ok(ArrayValue::zeros(arg!(args).unsafe_as_array())?.into_value())
}

pub fn ones(
    _ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    Ok(ArrayValue::ones(arg!(args).unsafe_as_array())?.into_value())
}

pub fn eye(
    _ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    Ok(ArrayValue::eye(arg!(args).unsafe_as_array())?.into_value())
}

pub fn head(
    _ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    arg!(args).unsafe_as_array().head()
}

pub fn tail(
    _ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    Ok(arg!(args).unsafe_as_array().tail()?.into_value())
}

pub fn cons(
    _ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    let element = arg!(args);
    Ok(arg!(args).unsafe_as_array().cons(element)?.into_value())
}

pub fn cons_end(
    _ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    let element = arg!(args);
    Ok(arg!(args).unsafe_as_array().cons_end(element)?.into_value())
}

pub fn take(
    _ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    let count = scalar_arg!(args).to_f64();
    let array = arg!(args).unsafe_as_array();

    if !count.is_finite() || count < 0.0 || count.fract() != 0.0 {
        return Err(Box::new(RuntimeErrorKind::EmptyArray));
    }

    Ok(array.take(count as usize)?.into_value())
}

pub fn drop(
    _ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    let count = scalar_arg!(args).to_f64();
    let array = arg!(args).unsafe_as_array();

    if !count.is_finite() || count < 0.0 || count.fract() != 0.0 {
        return Err(Box::new(RuntimeErrorKind::EmptyArray));
    }

    Ok(array.drop(count as usize)?.into_value())
}

pub fn element_at(
    _ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    let index = scalar_arg!(args).to_f64();
    let array = arg!(args).unsafe_as_array();

    if !index.is_finite() || index < 0.0 || index.fract() != 0.0 {
        return Err(Box::new(RuntimeErrorKind::EmptyArray));
    }

    array.element_at(index as usize)
}

pub fn contains(
    _ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    let needle = arg!(args);
    let array = arg!(args).unsafe_as_array();

    return_boolean!(array.contains(&needle)?)
}

pub fn vcat(
    _ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    let lhs = arg!(args).unsafe_as_array();
    let rhs = arg!(args).unsafe_as_array();
    Ok(lhs.vcat(rhs)?.into_value())
}

pub fn hcat(
    _ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    let lhs = arg!(args).unsafe_as_array();
    let rhs = arg!(args).unsafe_as_array();
    Ok(lhs.hcat(rhs)?.into_value())
}

pub fn reverse(
    _ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    Ok(arg!(args).unsafe_as_array().reverse()?.into_value())
}

pub fn unique(
    _ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    Ok(arg!(args).unsafe_as_array().unique()?.into_value())
}

pub fn intersperse(
    _ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    let separator = arg!(args);
    let array = arg!(args).unsafe_as_array();
    Ok(array.intersperse(separator)?.into_value())
}

pub fn map(
    ctx: &mut FfiContext,
    mut args: Args,
    return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    let callable = arg!(args);
    let array_arg = args.pop_front().unwrap();
    let element_type = array_element_type(&array_arg.type_);
    let mapped_element_type = array_return_element_type(return_type);
    let elements = array_arg.value.unsafe_as_array().vector_values()?;
    let mut mapped = Vec::with_capacity(elements.len());

    for element in elements {
        mapped.push(ctx.invoke_callable(
            callable.clone(),
            Args::from([Arg {
                value: element,
                type_: element_type.clone(),
                span: array_arg.span,
            }]),
            &mapped_element_type,
        )?);
    }

    Ok(Value::Array(crate::value::ArrayValue::from_values(mapped)))
}

pub fn map2(
    ctx: &mut FfiContext,
    mut args: Args,
    return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    let callable = arg!(args);
    let other_arg = args.pop_front().unwrap();
    let array_arg = args.pop_front().unwrap();
    let element_type = array_element_type(&array_arg.type_);
    let mapped_element_type = array_return_element_type(return_type);
    let elements = array_arg.value.unsafe_as_array().vector_values()?;
    let mut mapped = Vec::with_capacity(elements.len());

    for element in elements {
        mapped.push(ctx.invoke_callable(
            callable.clone(),
            Args::from([
                Arg {
                    value: other_arg.value.clone(),
                    type_: other_arg.type_.clone(),
                    span: other_arg.span,
                },
                Arg {
                    value: element,
                    type_: element_type.clone(),
                    span: array_arg.span,
                },
            ]),
            &mapped_element_type,
        )?);
    }

    Ok(Value::Array(crate::value::ArrayValue::from_values(mapped)))
}

pub fn filter(
    ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    let callable = arg!(args);
    let array_arg = args.pop_front().unwrap();
    let element_type = array_element_type(&array_arg.type_);
    let predicate_type = TypeScheme::concrete(Type::Boolean);
    let elements = array_arg.value.unsafe_as_array().vector_values()?;
    let mut filtered = Vec::with_capacity(elements.len());

    for element in elements {
        let keep = ctx.invoke_callable(
            callable.clone(),
            Args::from([Arg {
                value: element.clone(),
                type_: element_type.clone(),
                span: array_arg.span,
            }]),
            &predicate_type,
        )?;

        if keep.unsafe_as_bool() {
            filtered.push(element);
        }
    }

    Ok(Value::Array(crate::value::ArrayValue::from_values(
        filtered,
    )))
}

pub fn foldl(
    ctx: &mut FfiContext,
    mut args: Args,
    return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    let callable = arg!(args);
    let mut acc_arg = args.pop_front().unwrap();
    let array_arg = args.pop_front().unwrap();
    let element_type = array_element_type(&array_arg.type_);
    let elements = array_arg.value.unsafe_as_array().vector_values()?;

    for element in elements {
        acc_arg.value = ctx.invoke_callable(
            callable.clone(),
            Args::from([
                Arg {
                    value: acc_arg.value,
                    type_: acc_arg.type_.clone(),
                    span: acc_arg.span,
                },
                Arg {
                    value: element,
                    type_: element_type.clone(),
                    span: array_arg.span,
                },
            ]),
            return_type,
        )?;
    }

    Ok(acc_arg.value)
}

pub fn sort_by_key(
    ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    let callable_arg = args.pop_front().unwrap();
    let array_arg = args.pop_front().unwrap();
    let element_type = array_element_type(&array_arg.type_);
    let key_type = callable_return_type(&callable_arg.type_);
    let elements = array_arg.value.unsafe_as_array().vector_values()?;
    let mut keyed = Vec::with_capacity(elements.len());

    for element in elements {
        let key = ctx.invoke_callable(
            callable_arg.value.clone(),
            Args::from([Arg {
                value: element.clone(),
                type_: element_type.clone(),
                span: array_arg.span,
            }]),
            &key_type,
        )?;

        keyed.push(SortEntry {
            value: element,
            key: key.unsafe_as_quantity(),
        });
    }

    let sorted = merge_sort_entries(keyed)?
        .into_iter()
        .map(|entry| entry.value)
        .collect();

    Ok(Value::Array(ArrayValue::from_values(sorted)))
}

pub fn join(
    _ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    let array = arg!(args).unsafe_as_array();
    let separator = string_arg!(args);
    return_string!(owned = array.join(separator)?)
}

pub fn shape(
    _ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    Ok(Value::Array(arg!(args).unsafe_as_array().shape()))
}

pub fn reshape(
    _ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    let array = arg!(args).unsafe_as_array();
    let new_shape = arg!(args).unsafe_as_array();
    Ok(array.reshape(new_shape)?.into_value())
}

pub fn transpose(
    _ctx: &mut FfiContext,
    mut args: Args,
    _return_type: &TypeScheme,
) -> Result<Value, Box<RuntimeErrorKind>> {
    Ok(arg!(args).unsafe_as_array().transpose().into_value())
}
