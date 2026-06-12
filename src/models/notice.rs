use std::sync::atomic::{AtomicU64, Ordering};

static NOTICE_GEN: AtomicU64 = AtomicU64::new(0);

/// Information on what is going on in the app
#[derive(Clone, Debug, PartialEq)]
pub struct Notice {
    /// Monotonic generation counter
    pub r#gen: u64,
    /// Type of Information related
    pub kind: NoticeKind,
}

/// Type of Information
#[derive(Clone, Debug, PartialEq)]
pub enum NoticeKind {
    /// Synchronization with server
    /// with subject of Synchronization inside
    Sync(String),
    /// Information on miscelaneaous stuff
    /// with subject of information inside
    Info(String),
    /// Information on miscelaneaous error
    Error {
        /// Subject of error
        msg: String,
        /// Optional Information if operation related
        /// error should retry
        retry: Option<Retry>,
    },
}

/// Type of operations retryable
#[derive(Clone, Debug, PartialEq)]
pub enum Retry {
    /// Synchronization with server
    SyncApp,
}

impl Notice {
    /// Generate Synchronization Notice
    pub fn sync(s: impl Into<String>) -> Self {
        Self::new(NoticeKind::Sync(s.into()))
    }
    /// Generate Information Notice
    pub fn info(s: impl Into<String>) -> Self {
        Self::new(NoticeKind::Info(s.into()))
    }
    /// Generate Error Notice
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
