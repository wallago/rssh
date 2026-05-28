use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, Mutex},
};

use anyhow::Result;
use dioxus::{
    html::article,
    signals::{Signal, WritableExt},
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
                        articles: Load::Ready(feed_articles),
                    }
                })
                .collect::<Vec<FeedNode>>();
            CategoryNode {
                category,
                expanded: false,
                feeds: Load::Ready(cat_feeds),
            }
        })
        .collect()
}
