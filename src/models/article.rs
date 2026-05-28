use crate::{models::feed::Feed, prelude::Filter};
use miniflux_api::models::{Entry, EntryStatus};

#[derive(Clone, PartialEq, Debug)]
pub struct Article {
    pub id: String,
    pub feed: Feed,
    pub title: String,
    pub snippet: String,
    pub timestamp: String,
    pub url: String,
    pub content: String,
    pub is_read: bool,
    pub is_starred: bool,
}

impl Article {
    pub fn matches(&self, filter: Filter) -> bool {
        match filter {
            Filter::All => true,
            Filter::Unread => !self.is_read,
            Filter::Starred => self.is_starred,
            Filter::Archived => false,
        }
    }
}
