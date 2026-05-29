use crate::prelude::{FeedNode, string_to_color};

#[derive(Clone, PartialEq, Debug)]
pub struct Category {
    pub id: i64,
    pub label: String,
    pub color: String,
}

#[derive(Clone, PartialEq)]
pub struct CategoryNode {
    pub category: Category,
    pub expanded: bool,
    pub feeds: Option<Vec<FeedNode>>,
}
