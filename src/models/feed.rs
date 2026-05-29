use crate::{
    models::category::{self, Category},
    prelude::Article,
};

#[derive(Clone, PartialEq, Debug)]
pub struct Feed {
    pub id: i64,
    pub label: String,
    pub category: Category,
    pub error_count: i64,
}

#[derive(Clone, PartialEq)]
pub struct FeedNode {
    pub feed: Feed,
    pub expanded: bool,
    pub articles: Option<Vec<Article>>,
}
