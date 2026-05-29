use std::sync::atomic::{AtomicU64, Ordering};

static NOTICE_GEN: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, PartialEq)]
pub struct Notice {
    pub r#gen: u64,
    pub kind: NoticeKind,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NoticeKind {
    Sync(String),                                // spinner, no auto-dismiss
    Info(String),                                // 3s auto-dismiss
    Error { msg: String, retry: Option<Retry> }, // 6s auto-dismiss, optional action
}

#[derive(Clone, Debug, PartialEq)]
pub enum Retry {
    Refresh,
    // grow as needed: MarkRead(ArticleId), Star(ArticleId), …
}

impl Notice {
    pub fn sync(s: impl Into<String>) -> Self {
        Self::new(NoticeKind::Sync(s.into()))
    }
    pub fn info(s: impl Into<String>) -> Self {
        Self::new(NoticeKind::Info(s.into()))
    }
    pub fn error(msg: impl Into<String>, retry: Option<Retry>) -> Self {
        Self::new(NoticeKind::Error {
            msg: msg.into(),
            retry,
        })
    }
    fn new(kind: NoticeKind) -> Self {
        Self {
            r#gen: NOTICE_GEN.fetch_add(1, Ordering::Relaxed),
            kind,
        }
    }
}
