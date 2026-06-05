use crate::prelude::*;
use dioxus::signals::{Signal, WritableExt};

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
        .filter(|node| node.feeds.as_ref().is_some_and(|f| !f.is_empty()))
        .collect()
}

pub fn toggle_category(mut tree: Signal<Option<Vec<CategoryNode>>>, node: CategoryNode) {
    if let Some(cats) = tree.write().as_mut() {
        if let Some(cat) = cats.into_iter().find(|c| c.category.id == node.category.id) {
            tracing::debug!("toggle expanded category: {}", node.category.label);
            cat.expanded = !cat.expanded;
            return;
        }
    }
}

pub fn toggle_feed(mut tree: Signal<Option<Vec<CategoryNode>>>, node: FeedNode) {
    if let Some(cats) = tree.write().as_mut() {
        for c in cats.iter_mut() {
            if let Some(feed) = c
                .feeds
                .iter_mut()
                .flatten()
                .find(|f| f.feed.id == node.feed.id)
            {
                tracing::debug!("toggle expanded feed: {}", node.feed.label);
                feed.expanded = !feed.expanded;
                return;
            }
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

pub fn update_article(
    mut tree: Signal<Option<Vec<CategoryNode>>>,
    article_id: i64,
    apply: impl Fn(&mut Article),
) {
    if let Some(cats) = tree.write().as_mut() {
        for c in cats.iter_mut() {
            for feed in c.feeds.iter_mut().flatten() {
                if let Some(a) = feed
                    .articles
                    .iter_mut()
                    .flatten()
                    .find(|a| a.id == article_id)
                {
                    apply(a);
                    return;
                }
            }
        }
    }
}

pub fn article_by_id(cats: &[CategoryNode], id: i64) -> Option<Article> {
    cats.iter()
        .flat_map(|c| c.feeds.iter().flatten())
        .flat_map(|f| f.articles.iter().flatten())
        .find(|a| a.id == id)
        .cloned()
}

pub fn feed_by_id(cats: &[CategoryNode], id: i64) -> Option<Feed> {
    cats.iter()
        .flat_map(|c| c.feeds.iter().flatten())
        .find(|f| f.feed.id == id)
        .map(|f| f.feed.clone())
}

pub fn cat_by_id(cats: &[CategoryNode], id: i64) -> Option<Category> {
    cats.iter()
        .find(|c| c.category.id == id)
        .map(|c| c.category.clone())
}

pub fn feed_nav_ids(
    cats: &[CategoryNode],
    cat_id: i64,
    feed_id: i64,
    filter: Filter,
    keep: i64,
) -> Vec<i64> {
    iter_articles(cats.to_vec(), Some(cat_id), Some(feed_id), None)
        .filter(|a| a.matches(filter) || a.id == keep)
        .map(|a| a.id)
        .collect()
}
