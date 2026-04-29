use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub enum IntegerFmt {
    Dec,
    Hex,
    Oct,
    Bin,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub enum FloatFmt {
    Decimal,
    Scientific,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub enum StringFmt {
    DoubleQuote,
    SingleQuote,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Type
{
    Bool,
    U8, U16, U32, U64,
    I8, I16, I32, I64,
    F32, F64,
    String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct IntegerValue {
    pub value: i64,
    pub fmt: IntegerFmt,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct FloatValue {
    pub value: f64,
    pub fmt: FloatFmt,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct StringValue {
    pub value: String,
    pub fmt: StringFmt,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum Value {
    Integer(IntegerValue),
    Float(FloatValue),
    String(StringValue),
}
