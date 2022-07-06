use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::interpreter::{StoredValue, ValueKey};

use crate::sources::CodeArea;

pub type ArbitraryId = u16;
pub type SpecificId = u16;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Id {
    Specific(SpecificId),
    ArbitraryId(ArbitraryId),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Value {
    Int(isize),
    Float(f64),

    String(String),

    Bool(bool),

    Empty,

    Array(Vec<StoredValue>),
    Dict(HashMap<String, StoredValue>),
    Maybe(Option<Box<StoredValue>>),

    TypeIndicator(ValueType),
    Pattern(Pattern),

    Group(Id),
    TriggerFunc { start_group: Id },

    Macro(Macro),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Macro {
    pub func_id: usize,
    pub args: Vec<((String, CodeArea), Option<StoredValue>, Option<StoredValue>)>,
    pub ret_type: Box<StoredValue>,
}

impl Value {
    pub fn into_stored(self, area: CodeArea) -> StoredValue {
        StoredValue {
            value: self,
            def_area: area,
        }
    }
    pub fn get_type(&self) -> ValueType {
        match self {
            Value::Int(_) => ValueType::Int,
            Value::Float(_) => ValueType::Float,
            Value::String(_) => ValueType::String,
            Value::Bool(_) => ValueType::Bool,
            Value::Empty => ValueType::Empty,
            Value::Array(_) => ValueType::Array,
            Value::Dict(_) => ValueType::Dict,
            Value::Maybe(_) => ValueType::Maybe,
            Value::TypeIndicator(_) => ValueType::TypeIndicator,
            Value::Pattern(_) => ValueType::Pattern,
            Value::Group(_) => ValueType::Group,
            Value::TriggerFunc { .. } => ValueType::TriggerFunc,
            Value::Macro(_) => ValueType::Macro,
        }
    }
    pub fn to_str(&self) -> String {
        match self {
            Value::Int(v) => v.to_string(),
            Value::Float(v) => v.to_string(),
            Value::String(v) => v.to_string(),
            Value::Bool(v) => v.to_string(),
            Value::Empty => "()".into(),
            Value::Array(arr) => format!(
                "[{}]",
                arr.iter()
                    .map(|v| v.value.to_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Dict(map) => format!(
                "{{{}}}",
                map.iter()
                    .map(|(k, v)| format!("{}: {}", k, v.value.to_str()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Maybe(None) => "?".into(),
            Value::Maybe(Some(v)) => format!("{}?", v.value.to_str()),
            Value::TypeIndicator(typ) => typ.to_str(),
            Value::Pattern(p) => p.to_str(),
            Value::Group(_) => todo!(),
            Value::TriggerFunc { start_group } => todo!(),
            Value::Macro(Macro {
                func_id,
                args,
                ret_type,
            }) => {
                format!(
                    "({}) -> {} {{...}}",
                    args.iter()
                        .map(|((n, _), t, d)| {
                            format!(
                                "{}{}{}",
                                n,
                                if let Some(t) = t {
                                    format!(": {}", t.value.to_str())
                                } else {
                                    "".into()
                                },
                                if let Some(d) = d {
                                    format!(" = {}", d.value.to_str())
                                } else {
                                    "".into()
                                },
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", "),
                    ret_type.value.to_str(),
                )
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
pub enum ValueType {
    Int,
    Float,
    String,
    Bool,
    Empty,
    Array,
    Dict,
    Maybe,
    TypeIndicator,
    Pattern,
    Group,
    TriggerFunc,
    Macro,
    // more soon
}
impl ValueType {
    pub fn to_str(&self) -> String {
        format!(
            "@{}",
            match self {
                ValueType::Int => "int",
                ValueType::Float => "float",
                ValueType::String => "string",
                ValueType::Bool => "bool",
                ValueType::Empty => "empty",
                ValueType::Array => "array",
                ValueType::Dict => "dictionary",
                ValueType::Maybe => "maybe",
                ValueType::TypeIndicator => "type_indicator",
                ValueType::Pattern => "pattern",
                ValueType::Group => "group",
                ValueType::TriggerFunc => "trigger_function",
                ValueType::Macro => "macro",
            }
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Pattern {
    Any,
    Type(ValueType),
}
impl Pattern {
    pub fn to_str(&self) -> String {
        match self {
            Pattern::Any => "_".into(),
            Pattern::Type(t) => t.to_str(),
        }
    }
}

// ok so this is a temporary thing until we get builtins and i can replace this with _plus_ and such
pub mod value_ops {
    use super::super::error::RuntimeError;
    use super::super::interpreter::StoredValue;
    use super::Pattern;
    use super::Value;
    use super::ValueType;

    use crate::sources::CodeArea;

    pub fn equality(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Int(n1), Value::Float(n2)) => *n1 as f64 == *n2,
            (Value::Float(n1), Value::Int(n2)) => *n1 == *n2 as f64,

            (Value::Array(arr1), Value::Array(arr2)) => {
                if arr1.len() != arr2.len() {
                    false
                } else {
                    arr1.iter()
                        .zip(arr2)
                        .all(|(a, b)| equality(&a.value, &b.value))
                }
            }
            (Value::Dict(map1), Value::Dict(map2)) => {
                if map1.len() != map2.len() {
                    false
                } else {
                    for (k, a) in map1 {
                        match map2.get(k) {
                            Some(b) => {
                                if !equality(&a.value, &b.value) {
                                    return false;
                                }
                            }
                            None => return false,
                        }
                    }
                    true
                }
            }

            (Value::Maybe(None), Value::Maybe(None)) => true,
            (Value::Maybe(Some(a)), Value::Maybe(Some(b))) => equality(&a.value, &b.value),

            _ => a == b,
        }
    }

    pub fn matches_pat(val: &Value, pat: &Pattern) -> bool {
        match (val, pat) {
            (_, Pattern::Any) => true,
            (_, Pattern::Type(t)) => &val.get_type() == t,
        }
    }

    pub fn to_bool(a: &StoredValue) -> Result<bool, RuntimeError> {
        match &a.value {
            Value::Bool(b) => Ok(*b),
            _ => Err(RuntimeError::CannotConvert {
                a: a.clone(),
                to: ValueType::Bool,
            }),
        }
    }

    pub fn to_pat(a: &StoredValue) -> Result<Pattern, RuntimeError> {
        match &a.value {
            Value::TypeIndicator(typ) => Ok(Pattern::Type(*typ)),
            Value::Pattern(p) => Ok(p.clone()),
            _ => Err(RuntimeError::CannotConvert {
                a: a.clone(),
                to: ValueType::Bool,
            }),
        }
    }

    pub fn plus(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
    ) -> Result<StoredValue, RuntimeError> {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => Value::Int(*n1 + *n2),
            (Value::Int(n1), Value::Float(n2)) => Value::Float(*n1 as f64 + *n2),
            (Value::Float(n1), Value::Int(n2)) => Value::Float(*n1 + *n2 as f64),
            (Value::Float(n1), Value::Float(n2)) => Value::Float(*n1 + *n2),
            (Value::String(s1), Value::String(s2)) => Value::String(s1.clone() + s2),

            (Value::Array(arr1), Value::Array(arr2)) => {
                Value::Array(arr1.iter().chain(arr2).cloned().collect::<Vec<_>>())
            }

            _ => {
                return Err(RuntimeError::InvalidOperands {
                    a: a.clone(),
                    b: b.clone(),
                    op: "+".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }
    pub fn minus(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
    ) -> Result<StoredValue, RuntimeError> {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => Value::Int(*n1 - *n2),
            (Value::Int(n1), Value::Float(n2)) => Value::Float(*n1 as f64 - *n2),
            (Value::Float(n1), Value::Int(n2)) => Value::Float(*n1 - *n2 as f64),
            (Value::Float(n1), Value::Float(n2)) => Value::Float(*n1 - *n2),
            _ => {
                return Err(RuntimeError::InvalidOperands {
                    a: a.clone(),
                    b: b.clone(),
                    op: "-".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }
    pub fn mult(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
    ) -> Result<StoredValue, RuntimeError> {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => Value::Int(*n1 * *n2),
            (Value::Int(n1), Value::Float(n2)) => Value::Float(*n1 as f64 * *n2),
            (Value::Float(n1), Value::Int(n2)) => Value::Float(*n1 * *n2 as f64),
            (Value::Float(n1), Value::Float(n2)) => Value::Float(*n1 * *n2),

            (Value::Int(n), Value::String(s)) => {
                Value::String(s.repeat(if *n < 0 { 0 } else { *n as usize }))
            }
            (Value::String(s), Value::Int(n)) => {
                Value::String(s.repeat(if *n < 0 { 0 } else { *n as usize }))
            }
            _ => {
                return Err(RuntimeError::InvalidOperands {
                    a: a.clone(),
                    b: b.clone(),
                    op: "*".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }
    pub fn div(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
    ) -> Result<StoredValue, RuntimeError> {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => Value::Int(*n1 / *n2),
            (Value::Int(n1), Value::Float(n2)) => Value::Float(*n1 as f64 / *n2),
            (Value::Float(n1), Value::Int(n2)) => Value::Float(*n1 / *n2 as f64),
            (Value::Float(n1), Value::Float(n2)) => Value::Float(*n1 / *n2),
            _ => {
                return Err(RuntimeError::InvalidOperands {
                    a: a.clone(),
                    b: b.clone(),
                    op: "/".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }
    pub fn modulo(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
    ) -> Result<StoredValue, RuntimeError> {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => Value::Int(*n1 % *n2),
            (Value::Int(n1), Value::Float(n2)) => Value::Float(*n1 as f64 % *n2),
            (Value::Float(n1), Value::Int(n2)) => Value::Float(*n1 % *n2 as f64),
            (Value::Float(n1), Value::Float(n2)) => Value::Float(*n1 % *n2),
            _ => {
                return Err(RuntimeError::InvalidOperands {
                    a: a.clone(),
                    b: b.clone(),
                    op: "%".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }
    pub fn pow(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
    ) -> Result<StoredValue, RuntimeError> {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => {
                Value::Int((*n1 as f64).powf(*n2 as f64).floor() as isize)
            }
            (Value::Int(n1), Value::Float(n2)) => Value::Float((*n1 as f64).powf(*n2)),
            (Value::Float(n1), Value::Int(n2)) => Value::Float((*n1).powf(*n2 as f64)),
            (Value::Float(n1), Value::Float(n2)) => Value::Float(n1.powf(*n2)),
            _ => {
                return Err(RuntimeError::InvalidOperands {
                    a: a.clone(),
                    b: b.clone(),
                    op: "^".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }
    pub fn eq(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
    ) -> Result<StoredValue, RuntimeError> {
        Ok(Value::Bool(equality(&a.value, &b.value)).into_stored(area))
    }
    pub fn not_eq(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
    ) -> Result<StoredValue, RuntimeError> {
        Ok(Value::Bool(!equality(&a.value, &b.value)).into_stored(area))
    }
    pub fn greater(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
    ) -> Result<StoredValue, RuntimeError> {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => Value::Bool(*n1 > *n2),
            (Value::Int(n1), Value::Float(n2)) => Value::Bool(*n1 as f64 > *n2),
            (Value::Float(n1), Value::Int(n2)) => Value::Bool(*n1 > *n2 as f64),
            (Value::Float(n1), Value::Float(n2)) => Value::Bool(*n1 > *n2),
            _ => {
                return Err(RuntimeError::InvalidOperands {
                    a: a.clone(),
                    b: b.clone(),
                    op: ">".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }
    pub fn greater_eq(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
    ) -> Result<StoredValue, RuntimeError> {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => Value::Bool(*n1 >= *n2),
            (Value::Int(n1), Value::Float(n2)) => Value::Bool(*n1 as f64 >= *n2),
            (Value::Float(n1), Value::Int(n2)) => Value::Bool(*n1 >= *n2 as f64),
            (Value::Float(n1), Value::Float(n2)) => Value::Bool(*n1 >= *n2),
            _ => {
                return Err(RuntimeError::InvalidOperands {
                    a: a.clone(),
                    b: b.clone(),
                    op: ">=".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }
    pub fn lesser(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
    ) -> Result<StoredValue, RuntimeError> {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => Value::Bool(*n1 < *n2),
            (Value::Int(n1), Value::Float(n2)) => Value::Bool((*n1 as f64) < *n2),
            (Value::Float(n1), Value::Int(n2)) => Value::Bool(*n1 < *n2 as f64),
            (Value::Float(n1), Value::Float(n2)) => Value::Bool(*n1 < *n2),
            _ => {
                return Err(RuntimeError::InvalidOperands {
                    a: a.clone(),
                    b: b.clone(),
                    op: "<".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }
    pub fn lesser_eq(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
    ) -> Result<StoredValue, RuntimeError> {
        let value = match (&a.value, &b.value) {
            (Value::Int(n1), Value::Int(n2)) => Value::Bool(*n1 <= *n2),
            (Value::Int(n1), Value::Float(n2)) => Value::Bool(*n1 as f64 <= *n2),
            (Value::Float(n1), Value::Int(n2)) => Value::Bool(*n1 <= *n2 as f64),
            (Value::Float(n1), Value::Float(n2)) => Value::Bool(*n1 <= *n2),
            _ => {
                return Err(RuntimeError::InvalidOperands {
                    a: a.clone(),
                    b: b.clone(),
                    op: "<=".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }

    pub fn unary_negate(a: &StoredValue, area: CodeArea) -> Result<StoredValue, RuntimeError> {
        let value = match &a.value {
            Value::Int(n) => Value::Int(-n),
            Value::Float(n) => Value::Float(-n),
            _ => {
                return Err(RuntimeError::InvalidUnaryOperand {
                    a: a.clone(),
                    op: "-".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }
    pub fn unary_not(a: &StoredValue, area: CodeArea) -> Result<StoredValue, RuntimeError> {
        let value = match &a.value {
            Value::Bool(n) => Value::Bool(!n),
            _ => {
                return Err(RuntimeError::InvalidUnaryOperand {
                    a: a.clone(),
                    op: "-".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }
    pub fn is_op(
        a: &StoredValue,
        b: &StoredValue,
        area: CodeArea,
    ) -> Result<StoredValue, RuntimeError> {
        let value = match (&a.value, &b.value) {
            (a, Value::TypeIndicator(typ)) => Value::Bool(&a.get_type() == typ),
            (a, Value::Pattern(pat)) => Value::Bool(matches_pat(a, pat)),
            (_, _) => {
                return Err(RuntimeError::TypeMismatch {
                    v: b.clone(),
                    expected: "@type_indicator or @pattern".into(),
                    area,
                })
            }
        };
        Ok(value.into_stored(area))
    }
}
