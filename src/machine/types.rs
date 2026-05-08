use serde::{Serialize, Deserialize, Deserializer};

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

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
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

#[derive(Serialize, PartialEq, Debug, Clone)]
pub enum Value {
    Integer(IntegerValue),
    Float(FloatValue),
    String(StringValue),
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, IntoDeserializer};

        // Try deserializing as a plain value first
        struct ValueVisitor;

        impl<'de> de::Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a number, string, or tagged value (Integer/Float/String)")
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
                Ok(Value::Integer(IntegerValue {
                    value: v,
                    fmt: IntegerFmt::Dec,
                }))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
                Ok(Value::Integer(IntegerValue {
                    value: v as i64,
                    fmt: IntegerFmt::Dec,
                }))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
                Ok(Value::Float(FloatValue {
                    value: v,
                    fmt: FloatFmt::Decimal,
                }))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
                Ok(Value::String(StringValue {
                    value: v.to_string(),
                    fmt: StringFmt::DoubleQuote,
                }))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                // Tagged representation: e.g. { Integer: { value: 0, fmt: Dec } }
                let entry = map.next_entry::<String, serde_json::Value>()
                    .map_err(de::Error::custom)?;
                match entry {
                    Some((key, inner)) => match key.as_str() {
                        "Integer" => {
                            let v = inner
                                .get("value")
                                .and_then(|vv| vv.as_i64())
                                .ok_or_else(|| de::Error::custom("Integer value missing"))?;
                            let fmt = inner
                                .get("fmt")
                                .and_then(|ff| ff.as_str())
                                .map(|s| match s {
                                    "Dec" => IntegerFmt::Dec,
                                    "Hex" => IntegerFmt::Hex,
                                    "Oct" => IntegerFmt::Oct,
                                    "Bin" => IntegerFmt::Bin,
                                    _ => IntegerFmt::Dec,
                                })
                                .unwrap_or(IntegerFmt::Dec);
                            Ok(Value::Integer(IntegerValue { value: v, fmt }))
                        }
                        "Float" => {
                            let v = inner
                                .get("value")
                                .and_then(|vv| vv.as_f64())
                                .ok_or_else(|| de::Error::custom("Float value missing"))?;
                            let fmt = inner
                                .get("fmt")
                                .and_then(|ff| ff.as_str())
                                .map(|s| match s {
                                    "Decimal" => FloatFmt::Decimal,
                                    "Scientific" => FloatFmt::Scientific,
                                    _ => FloatFmt::Decimal,
                                })
                                .unwrap_or(FloatFmt::Decimal);
                            Ok(Value::Float(FloatValue { value: v, fmt }))
                        }
                        "String" => {
                            let v = inner
                                .get("value")
                                .and_then(|vv| vv.as_str())
                                .ok_or_else(|| de::Error::custom("String value missing"))?;
                            Ok(Value::String(StringValue {
                                value: v.to_string(),
                                fmt: StringFmt::DoubleQuote,
                            }))
                        }
                        _ => Err(de::Error::custom(format!(
                            "unknown Value tag: '{}'",
                            key
                        ))),
                    },
                    None => Err(de::Error::custom("empty value map")),
                }
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}
