use crate::prelude::{Article, Category, Feed};

#[derive(Clone, PartialEq)]
pub enum Load<T> {
    Idle,
    Loading,
    Ready(T),
    Failed(String),
}

#[derive(Clone, PartialEq)]
pub struct CategoryNode {
    pub category: Category,
    pub expanded: bool,
    pub feeds: Load<Vec<FeedNode>>,
}

#[derive(Clone, PartialEq)]
pub struct FeedNode {
    pub feed: Feed,
    pub expanded: bool,
    pub articles: Load<Vec<Article>>,
}

#[derive(Clone, PartialEq)]
pub struct ReaderState {
    pub id: String,
}
