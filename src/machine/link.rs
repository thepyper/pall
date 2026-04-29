use serde::{Deserialize, Serialize, Serializer};

#[derive(Deserialize, PartialEq, Debug, Clone)]
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
