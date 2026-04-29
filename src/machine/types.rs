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

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Value
{
    Integer(i64),
    Float(f64),
    String(String),
}
