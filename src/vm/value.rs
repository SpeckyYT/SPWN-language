use std::fmt::write;

use ahash::AHashMap;
use lasso::Spur;
use strum::EnumDiscriminants;

use super::builtins::Builtin;
// use super::builtins::{Builtin, BuiltinFn};
use super::interpreter::{FuncCoord, ValueKey, Vm};
use super::opcodes::FunctionID;
use crate::compiling::bytecode::Constant;
use crate::gd::ids::*;
use crate::sources::CodeArea;

#[derive(Debug, Clone, PartialEq)]
pub struct StoredValue {
    pub value: Value,
    pub area: CodeArea,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct ArgData {
    pub name: Spur,
    pub default: Option<ValueKey>,
    pub pattern: Option<ValueKey>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MacroCode {
    Normal {
        func: FuncCoord,
        args: Vec<ArgData>,
        captured: Vec<ValueKey>,
    },
    Builtin(Builtin),
}

#[derive(EnumDiscriminants, Debug, Clone, PartialEq)]
// `EnumDiscriminants` generates a new enum that is just the variant names without any data
// anything in `strum_discriminants` is applied to the `ValueType` enum
#[strum_discriminants(name(ValueType))]
#[strum_discriminants(derive(delve::EnumToStr))]
#[strum_discriminants(delve(rename_all = "lowercase"))]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),

    Array(Vec<ValueKey>),
    Dict(AHashMap<Spur, ValueKey>),

    Group(Id),
    Color(Id),
    Block(Id),
    Item(Id),

    Builtins,

    Range(i64, i64, usize), //start, end, step

    Maybe(Option<ValueKey>),
    Empty,
    Macro(MacroCode),

    TriggerFunction(Id),
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "@{}", <&ValueType as Into<&'static str>>::into(self))
    }
}

impl Value {
    pub fn get_type(&self) -> ValueType {
        self.into()
    }

    pub fn from_const(c: &Constant) -> Self {
        match c {
            Constant::Int(v) => Value::Int(*v),
            Constant::Float(v) => Value::Float(*v),
            Constant::String(v) => Value::String(v.clone()),
            Constant::Bool(v) => Value::Bool(*v),
            Constant::Id(c, v) => {
                let id = Id::Specific(*v);
                match c {
                    IDClass::Group => Value::Group(id),
                    IDClass::Color => Value::Color(id),
                    IDClass::Block => Value::Block(id),
                    IDClass::Item => Value::Item(id),
                }
            }
        }
    }

    pub fn runtime_display(&self, vm: &Vm) -> String {
        match self {
            Value::Int(n) => n.to_string(),
            Value::Float(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::String(s) => s.clone(),
            Value::Array(arr) => format!(
                "[{}]",
                arr.iter()
                    .map(|k| vm.memory[*k].value.runtime_display(vm))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Dict(d) => format!(
                "{{{}}}",
                d.iter()
                    .map(|(s, k)| format!(
                        "{}: {}",
                        vm.interner.borrow().resolve(s),
                        vm.memory[*k].value.runtime_display(vm)
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Group(id) => id.fmt("g"),
            Value::Color(id) => id.fmt("c"),
            Value::Block(id) => id.fmt("b"),
            Value::Item(id) => id.fmt("i"),
            Value::Builtins => "$".to_string(),
            Value::Range(n1, n2, s) => {
                if *s == 1 {
                    format!("{n1}..{n2}")
                } else {
                    format!("{n1}..{s}..{n2}")
                }
            }
            Value::Maybe(o) => match o {
                Some(k) => format!("({})?", vm.memory[*k].value.runtime_display(vm)),
                None => "?".into(),
            },
            Value::Empty => "()".into(),
            Value::Macro(MacroCode::Normal {
                func,
                args,
                captured,
            }) => format!(
                "({}) {{...}}",
                args.iter()
                    .map(|d| vm.resolve(&d.name))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Macro(MacroCode::Builtin(b)) => format!("<builtin: {b}>"),
            Value::TriggerFunction(_) => "!{{...}}".to_string(),
        }
    }
}
