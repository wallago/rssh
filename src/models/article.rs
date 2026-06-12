use crate::{models::feed::Feed, prelude::Filter};

/// RSS Article
#[derive(Clone, PartialEq, Debug)]
pub struct Article {
    /// ID
    pub id: i64,
    /// Feed related
    pub feed: Feed,
    /// Title
    pub title: String,
    /// Snippet
    pub snippet: String,
    /// Release Date
    pub timestamp: String,
    /// Link to Article
    pub url: String,
    /// Content
    pub content: String,
    /// Information if Article as been readed
    pub is_read: bool,
    /// Information if Article as been marked
    pub is_starred: bool,
}

impl Article {
    /// Fonction to know if Article match a given Filter
    pub fn matches(&self, filter: Filter) -> bool {
        match filter {
            Filter::All => true,
            Filter::Unread => !self.is_read,
            Filter::Starred => self.is_starred,
        }
    }
}
