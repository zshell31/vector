use crate::ast::{Function, FunctionArgument};
use crate::parse_grok::Error as GrokRuntimeError;
use crate::parse_grok_rules::Error as GrokStaticError;
use grok::Grok;
use parsing::{array, array::parse, query_string, ruby_hash, value::Value};
use percent_encoding::percent_decode;
use regex::Regex;
use std::{convert::TryFrom, convert::TryInto, ops::Deref, string::ToString};
use strum_macros::Display;
use tracing::error;

#[derive(Debug, Display, Clone)]
pub enum GrokFilter {
    Integer,
    IntegerExt,
    Number,
    NumberExt,
    Boolean(Option<Regex>),
    NullIf(String),
    Scale(f64),
    Json,
    Rubyhash,
    Querystring,
    Lowercase,
    Uppercase,
    Decodeuricomponent,
    Array(
        Option<(char, char)>,
        Option<String>,
        Box<Option<GrokFilter>>,
    ),
}

impl TryFrom<&Function> for GrokFilter {
    type Error = GrokStaticError;

    fn try_from(f: &Function) -> Result<Self, Self::Error> {
        match f.name.as_str() {
            "boolean" => {
                if f.args.is_some() && !f.args.as_ref().unwrap().is_empty() {
                    if let FunctionArgument::Arg(Value::Bytes(ref bytes)) =
                        f.args.as_ref().unwrap()[0]
                    {
                        let pattern = String::from_utf8_lossy(bytes);
                        Ok(GrokFilter::Boolean(Some(
                            Regex::new(pattern.deref()).map_err(|error| {
                                error!(message = "Error compiling regex", path = %pattern, %error);
                                GrokStaticError::InvalidFunctionArguments(f.name.clone())
                            })?,
                        )))
                    } else {
                        Err(GrokStaticError::InvalidFunctionArguments(f.name.clone()))
                    }
                } else {
                    Ok(GrokFilter::Boolean(None))
                }
            }
            "nullIf" => {
                if f.args.is_some() && !f.args.as_ref().unwrap().is_empty() {
                    if let FunctionArgument::Arg(ref null_value) = f.args.as_ref().unwrap()[0] {
                        return Ok(GrokFilter::NullIf(null_value.to_string_lossy()));
                    }
                }
                Err(GrokStaticError::InvalidFunctionArguments(f.name.clone()))
            }
            "scale" => {
                if f.args.is_some() && !f.args.as_ref().unwrap().is_empty() {
                    let scale_factor = match f.args.as_ref().unwrap()[0] {
                        FunctionArgument::Arg(Value::Integer(scale_factor)) => scale_factor as f64,
                        FunctionArgument::Arg(Value::Float(scale_factor)) => scale_factor,
                        _ => return Err(GrokStaticError::InvalidFunctionArguments(f.name.clone())),
                    };
                    return Ok(GrokFilter::Scale(scale_factor));
                }
                Err(GrokStaticError::InvalidFunctionArguments(f.name.clone()))
            }
            "array" => {
                let args_len = f.args.as_ref().map_or(0, |args| args.len());

                let mut delimiter = None;
                let mut value_filter = None;
                let mut brackets = None;
                if args_len == 1 {
                    match &f.args.as_ref().unwrap()[0] {
                        FunctionArgument::Arg(Value::Bytes(ref bytes)) => {
                            delimiter = Some(String::from_utf8_lossy(bytes).to_string());
                        }
                        FunctionArgument::Function(f) => {
                            value_filter = Some(GrokFilter::try_from(f)?)
                        }
                        _ => return Err(GrokStaticError::InvalidFunctionArguments(f.name.clone())),
                    }
                } else if args_len == 2 {
                    match (&f.args.as_ref().unwrap()[0], &f.args.as_ref().unwrap()[1]) {
                        (
                            FunctionArgument::Arg(Value::Bytes(ref brackets_b)),
                            FunctionArgument::Arg(Value::Bytes(ref delimiter_b)),
                        ) => {
                            brackets = Some(String::from_utf8_lossy(brackets_b).to_string());
                            delimiter = Some(String::from_utf8_lossy(delimiter_b).to_string());
                        }
                        (
                            FunctionArgument::Arg(Value::Bytes(ref delimiter_b)),
                            FunctionArgument::Function(f),
                        ) => {
                            delimiter = Some(String::from_utf8_lossy(delimiter_b).to_string());
                            value_filter = Some(GrokFilter::try_from(f)?);
                        }
                        _ => return Err(GrokStaticError::InvalidFunctionArguments(f.name.clone())),
                    }
                } else if args_len == 3 {
                    match (
                        &f.args.as_ref().unwrap()[0],
                        &f.args.as_ref().unwrap()[1],
                        &f.args.as_ref().unwrap()[2],
                    ) {
                        (
                            FunctionArgument::Arg(Value::Bytes(ref brackets_b)),
                            FunctionArgument::Arg(Value::Bytes(ref delimiter_b)),
                            FunctionArgument::Function(f),
                        ) => {
                            brackets = Some(String::from_utf8_lossy(brackets_b).to_string());
                            delimiter = Some(String::from_utf8_lossy(delimiter_b).to_string());
                            value_filter = Some(GrokFilter::try_from(f)?);
                        }
                        _ => return Err(GrokStaticError::InvalidFunctionArguments(f.name.clone())),
                    }
                } else if args_len > 3 {
                    return Err(GrokStaticError::InvalidFunctionArguments(f.name.clone()));
                }

                let brackets = match brackets {
                    Some(b) if b.len() == 1 => {
                        let mut chars = b.chars();
                        Some((chars.nth(0).unwrap(), chars.nth(0).unwrap()))
                    }
                    Some(b) if b.len() == 2 => {
                        let mut chars = b.chars();
                        Some((chars.nth(0).unwrap(), chars.nth(1).unwrap()))
                    }
                    None => None,
                    _ => {
                        return Err(GrokStaticError::InvalidFunctionArguments(f.name.clone()));
                    }
                };

                Ok(GrokFilter::Array(
                    brackets,
                    delimiter,
                    Box::new(value_filter),
                ))
            }
            "integer" => Ok(GrokFilter::Integer),
            "integerExt" => Ok(GrokFilter::IntegerExt),
            "number" => Ok(GrokFilter::Number),
            "numberExt" => Ok(GrokFilter::NumberExt),
            "json" => Ok(GrokFilter::Json),
            "rubyhash" => Ok(GrokFilter::Rubyhash),
            "querystring" => Ok(GrokFilter::Querystring),
            "lowercase" => Ok(GrokFilter::Lowercase),
            "uppercase" => Ok(GrokFilter::Uppercase),
            "decodeuricomponent" => Ok(GrokFilter::Decodeuricomponent),
            _ => Err(GrokStaticError::UnknownFilter(f.name.clone())),
        }
    }
}

pub fn apply_filter(value: &Value, filter: &GrokFilter) -> Result<Value, GrokRuntimeError> {
    match filter {
        GrokFilter::Integer => match value {
            Value::Bytes(v) => Ok(String::from_utf8_lossy(v)
                .parse::<i64>()
                .map_err(|_e| {
                    GrokRuntimeError::FailedToApplyFilter(
                        filter.to_string(),
                        value.to_string_lossy(),
                    )
                })?
                .into()),
            _ => Err(GrokRuntimeError::FailedToApplyFilter(
                filter.to_string(),
                value.to_string_lossy(),
            )),
        },
        GrokFilter::IntegerExt => match value {
            Value::Bytes(v) => Ok(String::from_utf8_lossy(v)
                .parse::<f64>()
                .map_err(|_e| {
                    GrokRuntimeError::FailedToApplyFilter(
                        filter.to_string(),
                        value.to_string_lossy(),
                    )
                })
                .map(|f| (f as i64).into())
                .map_err(|_e| {
                    GrokRuntimeError::FailedToApplyFilter(
                        filter.to_string(),
                        value.to_string_lossy(),
                    )
                })?),
            _ => Err(GrokRuntimeError::FailedToApplyFilter(
                filter.to_string(),
                value.to_string_lossy(),
            )),
        },
        GrokFilter::Number | GrokFilter::NumberExt => match value {
            Value::Bytes(v) => Ok(String::from_utf8_lossy(v)
                .parse::<f64>()
                .map_err(|_e| {
                    GrokRuntimeError::FailedToApplyFilter(
                        filter.to_string(),
                        value.to_string_lossy(),
                    )
                })?
                .into()),
            _ => Err(GrokRuntimeError::FailedToApplyFilter(
                filter.to_string(),
                value.to_string_lossy(),
            )),
        },
        GrokFilter::Boolean(true_pattern) => match value {
            Value::Bytes(v) => {
                let is_true = match true_pattern {
                    Some(true_pattern) => {
                        true_pattern.is_match(String::from_utf8_lossy(v).as_ref())
                    }
                    None => "true".eq_ignore_ascii_case(String::from_utf8_lossy(v).as_ref()),
                };
                Ok(is_true.into())
            }
            _ => Err(GrokRuntimeError::FailedToApplyFilter(
                filter.to_string(),
                value.to_string_lossy(),
            )),
        },
        GrokFilter::NullIf(null_value) => match value {
            Value::Bytes(_) => {
                if value.to_string_lossy() == *null_value {
                    Ok(Value::Null)
                } else {
                    Ok(value.to_owned())
                }
            }
            _ => Err(GrokRuntimeError::FailedToApplyFilter(
                filter.to_string(),
                value.to_string_lossy(),
            )),
        },
        GrokFilter::Scale(scale_factor) => match value {
            Value::Integer(v) => Ok(Value::Float((*v as f64) * scale_factor)),
            Value::Float(v) => Ok(Value::Float(*v * scale_factor)),
            Value::Bytes(v) => Ok(Value::Float(
                String::from_utf8_lossy(v).parse::<f64>().map_err(|_e| {
                    GrokRuntimeError::FailedToApplyFilter(
                        filter.to_string(),
                        value.to_string_lossy(),
                    )
                })? * scale_factor,
            )),
            _ => Err(GrokRuntimeError::FailedToApplyFilter(
                filter.to_string(),
                value.to_string_lossy(),
            )),
        },
        GrokFilter::Json => match value {
            Value::Bytes(bytes) => serde_json::from_slice::<'_, serde_json::Value>(bytes.as_ref())
                .map_err(|_e| {
                    GrokRuntimeError::FailedToApplyFilter(
                        filter.to_string(),
                        value.to_string_lossy(),
                    )
                })
                .map(|v| v.into()),
            _ => Err(GrokRuntimeError::FailedToApplyFilter(
                filter.to_string(),
                value.to_string_lossy(),
            )),
        },
        GrokFilter::Rubyhash => match value {
            Value::Bytes(bytes) => ruby_hash::parse(String::from_utf8_lossy(&bytes).as_ref())
                .map_err(|_e| {
                    GrokRuntimeError::FailedToApplyFilter(
                        filter.to_string(),
                        value.to_string_lossy(),
                    )
                }),
            _ => Err(GrokRuntimeError::FailedToApplyFilter(
                filter.to_string(),
                value.to_string_lossy(),
            )),
        },
        GrokFilter::Querystring => match value {
            Value::Bytes(bytes) => query_string::parse(bytes).map_err(|_e| {
                GrokRuntimeError::FailedToApplyFilter(filter.to_string(), value.to_string_lossy())
            }),
            _ => Err(GrokRuntimeError::FailedToApplyFilter(
                filter.to_string(),
                value.to_string_lossy(),
            )),
        },
        GrokFilter::Lowercase => match value {
            Value::Bytes(bytes) => Ok(String::from_utf8_lossy(&bytes).to_lowercase().into()),
            _ => Err(GrokRuntimeError::FailedToApplyFilter(
                filter.to_string(),
                value.to_string_lossy(),
            )),
        },
        GrokFilter::Uppercase => match value {
            Value::Bytes(bytes) => Ok(String::from_utf8_lossy(&bytes).to_uppercase().into()),
            _ => Err(GrokRuntimeError::FailedToApplyFilter(
                filter.to_string(),
                value.to_string_lossy(),
            )),
        },
        GrokFilter::Decodeuricomponent => match value {
            Value::Bytes(bytes) => Ok(percent_decode(bytes).decode_utf8_lossy().to_string().into()),
            _ => Err(GrokRuntimeError::FailedToApplyFilter(
                filter.to_string(),
                value.to_string_lossy(),
            )),
        },
        GrokFilter::Array(brackets, delimiter, value_filter) => match value {
            Value::Bytes(bytes) => array::parse(
                String::from_utf8_lossy(&bytes).as_ref(),
                brackets.to_owned(),
                delimiter.as_ref().map(|s| s.as_str()),
            )
            .map_err(|_e| {
                GrokRuntimeError::FailedToApplyFilter(filter.to_string(), value.to_string_lossy())
            })
            .and_then(|values| {
                if let Some(value_filter) = value_filter.deref() {
                    return values
                        .iter()
                        .map(|v| apply_filter(v, value_filter.deref()))
                        .collect::<Result<Vec<Value>, _>>()
                        .map(|v| Value::from(v));
                }
                Ok(values.into())
            }),
            _ => Err(GrokRuntimeError::FailedToApplyFilter(
                filter.to_string(),
                value.to_string_lossy(),
            )),
        },
    }
}
