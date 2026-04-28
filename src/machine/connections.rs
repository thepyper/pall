use serde::{Serialize, Deserialize};

use super::types::Type;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Link
{
    pub id: String,
    pub output: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Input
{
    pub r#type: Type,
    pub link: Option<Link>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Output
{
    pub r#type: Type,
}
