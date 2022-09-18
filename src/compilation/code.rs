use std::ops::Index;

use ahash::AHashMap;
use ariadne::Source;
use slotmap::SlotMap;

use super::compiler::Constant;
use super::operators::Operator;
use crate::compilation::compiler::URegister;
use crate::regex_color_replace;
use crate::sources::{CodeSpan, SpwnSource};
use crate::vm::interpreter::{BuiltinKey, TypeKey};
use crate::vm::types::CustomType;

macro_rules! wrappers {
    ($($n:ident($t:ty))*) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct $n(pub $t);

            impl<T> Index<$n> for URegister<T> {
                type Output = T;

                fn index(&self, index: $n) -> &Self::Output {
                    &self.reg[index.0 as usize]
                }
            }
            impl From<$t> for $n {
                fn from(n: $t) -> Self {
                    $n(n)
                }
            }
        )*
    };
}
wrappers! {
    InstrNum(u16)

    VarID(u16)
    ConstID(u16)
    KeysID(u16)
    MemberID(u16)
    MacroBuildID(u16)
    ImportID(u16)
}

pub struct BytecodeFunc {
    pub instructions: Vec<(Instruction, CodeSpan)>,
    pub arg_ids: Vec<VarID>,
    pub capture_ids: Vec<VarID>,
    pub inner_ids: Vec<VarID>,
}

pub struct Code<'a> {
    pub source: SpwnSource,

    pub const_register: URegister<Constant>,
    pub keys_register: URegister<Vec<String>>,
    pub member_register: URegister<String>,
    pub import_register: URegister<SpwnSource>,
    #[allow(clippy::type_complexity)]
    pub macro_build_register: URegister<(usize, Vec<(String, bool, bool)>)>,

    pub var_count: usize,

    pub funcs: Vec<BytecodeFunc>,

    pub types: SlotMap<TypeKey, CustomType>,
    pub type_keys: AHashMap<String, TypeKey>,
    pub bltn: &'a AHashMap<String, BuiltinKey>,
}

impl<'a> Code<'a> {
    pub fn new(source: SpwnSource, bltn: &'a AHashMap<String, BuiltinKey>) -> Self {
        Self {
            source,
            const_register: URegister::new(),
            keys_register: URegister::new(),
            macro_build_register: URegister::new(),
            member_register: URegister::new(),

            var_count: 0,
            funcs: vec![],
            //modules: AHashMap::default(),
            import_register: URegister::new(),

            types: SlotMap::default(),
            type_keys: AHashMap::default(),
            bltn,
        }
    }

    #[cfg(debug_assertions)]
    pub fn debug(&self) {
        let mut debug_str = String::new();
        use std::fmt::Write;

        for (i, f) in self.funcs.iter().enumerate() {
            writeln!(
                &mut debug_str,
                "================== Func {} ================== arg_ids: {:?}, capture_ids: {:?}, inner_ids: {:?}",
                i,
                f.arg_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
                f.capture_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
                f.inner_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
            )
            .unwrap();
            for (i, (instr, _)) in f.instructions.iter().enumerate() {
                writeln!(
                    &mut debug_str,
                    "{}\t{:?}\t\t{}",
                    i,
                    instr,
                    ansi_term::Color::Green.bold().paint(match instr {
                        Instruction::LoadConst(c) => format!("{:?}", self.const_register[*c]),
                        Instruction::BuildDict(k) => format!("{:?}", self.keys_register[*k]),
                        Instruction::Member(k) => format!("{}", self.member_register[*k]),
                        Instruction::CallBuiltin(k) =>
                            format!("{}", self.bltn.iter().find(|(_, b)| b == &k).unwrap().0),
                        Instruction::BuildMacro(b) =>
                            format!("{:?}", self.macro_build_register[*b]),
                        Instruction::CallOp(op) => format!("{}", op.to_str()),
                        _ => "".into(),
                    })
                )
                .unwrap();
            }
        }

        regex_color_replace!(
            debug_str,
            r"ConstID\(([^)]*)\)", "const $1", Yellow
            r"VarID\(([^)]*)\)", "var $1", Yellow
            r"InstrNum\(([^)]*)\)", "$1", Yellow
            r"KeysID\(([^)]*)\)", "dict keys $1", Yellow
            r"MacroBuildID\(([^)]*)\)", "macro build $1", Yellow
            r"MemberID\(([^)]*)\)", "member $1", Yellow
            r"BuiltinKey\(([^)]*)\)", "$1", Yellow
            //r"CallOp\(([^)]*)\)", "$1", Yellow
        );

        println!("{}", debug_str);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct InstrPos {
    pub func: usize,
    pub idx: usize,
}

#[derive(Clone, Debug)]
pub enum Instruction {
    LoadConst(ConstID),

    CallOp(Operator),

    LoadVar(VarID),
    SetVar(VarID),
    CreateVar(VarID),

    BuildArray(InstrNum),
    BuildDict(KeysID),

    Jump(InstrNum),
    JumpIfFalse(InstrNum),
    UnwrapOrJump(InstrNum),

    PopTop,
    PushEmpty,

    WrapMaybe,
    PushNone,

    TriggerFuncCall,
    PushTriggerFn,

    Print,

    ToIter,
    IterNext(InstrNum),

    /// implements members on a type (pops 2 elements from the stack; type and members)
    Impl,

    PushAnyPattern,
    BuildMacro(MacroBuildID),
    Call(InstrNum),
    CallBuiltin(BuiltinKey),
    Return,

    Index,
    Member(MemberID),
    Associated(MemberID),
    TypeOf,

    YeetContext,
    EnterArrowStatement(InstrNum),
    EnterTriggerFunction(InstrNum),

    /// makes gd object data structure from last n elements on the stack
    BuildObject(InstrNum),
    BuildTrigger(InstrNum),
    AddObject,

    /// creates an instance of a type (pops 2 elements from the stack; type and fields)
    BuildInstance,
    PushBuiltins,
    Import(ImportID),
}

impl Instruction {
    pub fn modify_num(&mut self, n: u16) {
        match self {
            Self::BuildArray(num)
            | Self::Jump(num)
            | Self::JumpIfFalse(num)
            | Self::EnterArrowStatement(num)
            | Self::EnterTriggerFunction(num)
            | Self::IterNext(num)
            | Self::BuildObject(num)
            | Self::BuildTrigger(num)
            | Self::UnwrapOrJump(num) => num.0 = n,
            _ => panic!("can't modify number of variant that doesnt hold nubere rf  v 🤓🤓🤓"),
        }
    }
}
