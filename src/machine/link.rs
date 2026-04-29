use serde::{Deserialize as SerdeDeserialize, Deserializer, Serialize, Serializer};

#[derive(PartialEq, Debug, Clone)]
pub struct Link {
    pub id: String,
    pub output: String,
}

impl Serialize for Link {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}.{}", self.id, self.output))
    }
}

impl<'de> SerdeDeserialize<'de> for Link {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        super::parser::parse_link(&raw)
            .map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}
