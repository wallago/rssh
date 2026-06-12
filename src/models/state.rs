use crate::prelude::{CategoryNode, Filter, Notice, Server};
use dioxus::prelude::*;

/// Miniflux client + local DB, initialized lazily on first access
pub static SERVER: GlobalSignal<Server> = Signal::global(Server::init);

/// The category ▸ feed ▸ article tree shown on Home (None until first load)
pub static TREE: GlobalSignal<Option<Vec<CategoryNode>>> = Signal::global(|| None);

/// Current toast
pub static NOTICE: GlobalSignal<Option<Notice>> = Signal::global(|| None);

/// Active article filter
pub static FILTER: GlobalSignal<Filter> = Signal::global(|| Filter::Unread);
