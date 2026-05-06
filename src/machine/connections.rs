use serde::{Deserialize, Serialize};
use super::link::Link;
use super::types::Type;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Input {
    pub r#type: Type,
    pub link: Option<Link>,
    #[serde(default)]
    pub output: bool,
}


