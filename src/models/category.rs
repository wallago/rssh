use crate::prelude::string_to_color;

#[derive(Clone, PartialEq, Debug)]
pub struct Category {
    pub id: String,
    pub label: String,
    pub color: String,
}
