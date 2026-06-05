use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, Mutex},
};

use anyhow::Result;
use dioxus::{
    html::{article, filter},
    signals::{ReadSignal, Signal, WritableExt},
};
use futures::future::join_all;
use miniflux_api::{ApiError, MinifluxApi};
use reqwest::Client;
use rusqlite::{Connection, params};

use crate::prelude::*;

pub fn build_tree(
    cats: Vec<Category>,
    feeds: Vec<Feed>,
    articles: Vec<Article>,
) -> Vec<CategoryNode> {
    cats.into_iter()
        .map(|category| {
            let cat_feeds = feeds
                .clone()
                .into_iter()
                .filter(|f| f.category.id == category.id)
                .map(|feed| {
                    let feed_articles = articles
                        .clone()
                        .into_iter()
                        .filter(|a| a.feed.id == feed.id)
                        .collect();
                    FeedNode {
                        feed,
                        expanded: false,
                        articles: Some(feed_articles),
                    }
                })
                .collect::<Vec<FeedNode>>();
            CategoryNode {
                category,
                expanded: false,
                feeds: Some(cat_feeds),
            }
        })
        .collect()
}

pub fn toggle_category(mut tree: Signal<Option<Vec<CategoryNode>>>, node: CategoryNode) {
    if let Some(cats) = tree.write().as_mut() {
        if let Some(cat) = cats.into_iter().find(|c| c.category.id == node.category.id) {
            tracing::debug!("toggle expanded category: {}", node.category.label);
            cat.expanded = !cat.expanded;
        }
    }
}

pub fn toggle_feed(mut tree: Signal<Option<Vec<CategoryNode>>>, mut node: FeedNode) {
    if let Some(cats) = tree.write().as_mut() {
        if let Some(feed) = cats
            .iter_mut()
            .filter_map(|c| c.feeds.as_mut())
            .flatten()
            .find(|f| f.feed.id == node.feed.id)
        {
            tracing::debug!("toggle expanded feed: {}", node.feed.label);
            feed.expanded = !feed.expanded;
        }
    }
}

pub fn iter_articles(
    cats: Vec<CategoryNode>,
    cat_id: Option<i64>,
    feed_id: Option<i64>,
    filter: Option<Filter>,
) -> impl Iterator<Item = Article> {
    cats.into_iter()
        .filter(move |c| {
            if let Some(id) = cat_id {
                c.category.id == id
            } else {
                true
            }
        })
        .flat_map(|c| c.feeds.unwrap_or(Vec::new()))
        .filter(move |f| {
            if let Some(id) = feed_id {
                f.feed.id == id
            } else {
                true
            }
        })
        .flat_map(|f| f.articles.unwrap_or(Vec::new()))
        .filter(move |a| {
            if let Some(filter) = filter {
                a.matches(filter)
            } else {
                true
            }
        })
}
