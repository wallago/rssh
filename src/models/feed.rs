use crate::{models::category::Category, prelude::Article};

/// RSS Feed is a source of articles
#[derive(Clone, PartialEq, Debug)]
pub struct Feed {
    /// ID
    pub id: i64,
    /// Title
    pub label: String,
    /// Category related
    pub category: Category,
    /// Information of how many error as been encountered
    pub error_count: i64,
}

/// FeedNode is an helper to organized content fo Feed
#[derive(Clone, PartialEq)]
pub struct FeedNode {
    /// Feed related
    pub feed: Feed,
    /// Information if Category as been expanded
    pub expanded: bool,
    /// Articles related
    pub articles: Option<Vec<Article>>,
}
