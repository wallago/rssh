use crate::{models::feed::Feed, prelude::Filter};
use chrono::Utc;

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

    /// Human-friendly age of the article, e.g. "44m", "2h", "3d".
    /// Falls back to the date (YYYY-MM-DD) for anything older than a year,
    /// or to the raw value if it can't be parsed.
    pub fn relative_time(&self) -> String {
        let Ok(published) = chrono::DateTime::parse_from_rfc3339(&self.timestamp) else {
            return self.timestamp.clone();
        };
        let secs = (Utc::now() - published.with_timezone(&Utc))
            .num_seconds()
            .max(0);
        match secs {
            s if s < 60 => "now".to_string(),
            s if s < 3_600 => format!("{}m", s / 60),
            s if s < 86_400 => format!("{}h", s / 3_600),
            s if s < 31_536_000 => format!("{}d", s / 86_400),
            _ => self
                .timestamp
                .get(0..10)
                .unwrap_or(&self.timestamp)
                .to_string(),
        }
    }
}
