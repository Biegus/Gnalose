//structs represeting state after being parsed

use crate::utility::*;
use derive_new::*;

use derive_new;
use std::fmt::{self, Display};
use subenum::subenum;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RValue(pub usize);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArrayRef(pub usize);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlagRef(pub usize);

#[derive(Debug, Clone, Copy, derive_new::new, PartialEq)]
pub struct ArrayElement {
    pub array_ref: ArrayRef,
    pub index: IValue,
}


#[subenum(VValue, IValue)]
#[enum_unwrapper::unique_try_froms]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AValue {
    #[subenum(IValue)]
    LValue(i32),
    #[subenum(VValue, IValue)]
    RValue(RValue),
    #[subenum(VValue)]
    ArrayElement(ArrayElement),
}

impl IValue {
    pub fn as_avalue(&self) -> AValue {
        return match self {
            &IValue::RValue(rf) => AValue::RValue(rf),
            &IValue::LValue(vl) => AValue::LValue(vl),
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConditionType {
    Equal,
    NotEqual,
    Less,
    Greater,
    LessOrEqual,
    GreaterOrEqual,
}

//TODO: array suppport
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Define(RValue),
    DefineArray(ArrayRef),
    Undefine(RValue),
    Read(VValue),
    Print(AValue),
    PrintASCII(AValue),
    Subtract(AValue, VValue),
    Add(AValue, VValue),
    If(AValue, AValue, ConditionType),
    Fi,
    Mark(FlagRef),
    Unmark(FlagRef),
    Pin(FlagRef),
    Goto,
    UndefineArray(ArrayRef),
}

#[derive(new, Debug)]
pub struct OpLine {
    pub op: Op,
    pub line_num: usize,
    pub line_text: String,
}
#[derive(Default, new, Debug)]
pub struct Representation {
    pub variables_names: Vec<String>,
    pub array_names: Vec<(String, usize)>,
    pub flags_names: Vec<String>,
    pub ops: Vec<OpLine>,
}

impl Representation {
    pub fn get_flag(&self, t: &str) -> Option<FlagRef> {
        return Some(FlagRef(self.flags_names.iter().find_i(t)?));
    }
    pub fn get_variable(&self, t: &str) -> Option<RValue> {
        return Some(RValue(self.variables_names.iter().find_i(t)?));
    }
    pub fn get_array(&self, t: &str) -> Option<ArrayRef> {
        return Some(ArrayRef(self.array_names.iter().position(|e| e.0 == t)?));
    }
    pub fn get_flag_name(&self, id: FlagRef) -> String {
        return self.flags_names[id.0].clone();
    }
    pub fn get_variable_name(&self, id: RValue) -> String {
        return self.variables_names[id.0].clone();
    }
    pub fn get_array_name(&self, id: ArrayRef) -> String {
        return self.array_names[id.0].0.clone();
    }
    pub fn get_array_size(&self, id: ArrayRef) -> usize {
        return self.array_names[id.0].1.clone();
    }
}

#[derive(Debug)]
pub enum NameType {
    Variable,
    Array,
    Flag,
}

#[derive(Debug)]
pub enum OpParsingError {
    NameUsedTwice(String, NameType, NameType),
    NotDefinedVariable(String, NameType),
    InvalidStructure,
    DoubleLabel(String),
}

#[derive(Debug)]
pub enum AllowedKind {
    Name,
    VValue,
    AValue,
    ArrayRef,
}

#[derive(Debug, PartialEq)]
pub enum NameOrNumber {
    String(String),
    Number(i32),
}

#[derive(Debug, PartialEq)]
pub enum HigherToken {
    Name(String),
    Array(String, NameOrNumber),
    Literal(i32),
}
impl Display for OpParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpParsingError::InvalidStructure => write!(f, "The line is not mathing any correct command"),
            OpParsingError::NameUsedTwice(name, type_a, type_b) => {
                write!(f, "The name \"{}\" is used both for {:?} and {:?}", name, type_a, type_b)
            }

            OpParsingError::NotDefinedVariable(name, type_a) => {
                write!(f, "\"{}\" is not defined as {:?} at this point", name, type_a)
            }
            OpParsingError::DoubleLabel(label) => {
                write!(f, "label \"{}\" was defined twice", label)
            }
        }
    }
}

impl HigherToken {
    pub fn try_to_name_ref(&self) -> Option<&str> {
        if let HigherToken::Name(name) = self {
            return Some(name.as_str());
        }
        return None;
    }
}

impl AllowedKind {
    pub fn check(&self, t: &HigherToken) -> bool {
        match *self {
            Self::Name => matches!(t, HigherToken::Name(_)),
            Self::VValue => matches!(t, HigherToken::Name(_)) || matches!(t, HigherToken::Array(_, _)),
            Self::AValue => true,
            Self::ArrayRef => matches!(t, HigherToken::Array(_, _)),
        }
    }
}
