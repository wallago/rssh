use dioxus::prelude::*;

use crate::{
    models::filter::Filter,
    prelude::{Article, Category, CategoryNode, Feed, FeedNode},
};

fn iter_articles(
    cats: &[CategoryNode],
    cat_id: Option<i64>,
    feed_id: Option<i64>,
    filter: Option<Filter>,
) -> impl Iterator<Item = &Article> {
    cats.iter()
        .filter(move |c| cat_id.is_none_or(|id| c.category.id == id))
        .flat_map(|c| c.feeds.iter().flatten())
        .filter(move |f| feed_id.is_none_or(|id| f.feed.id == id))
        .flat_map(|f| f.articles.iter().flatten())
        .filter(move |a| filter.is_none_or(|fl| a.matches(fl)))
}

/// Operations on the global category ▸ feed ▸ article tree
pub trait Tree {
    /// Rebuild the tree from flat DB rows (drops empty categories, collapses everything)
    fn build(&self, cats: Vec<Category>, feeds: Vec<Feed>, articles: Vec<Article>);
    /// Expand/collapse a category
    fn toggle_category(&self, node: CategoryNode);
    /// Expand/collapse a feed
    fn toggle_feed(&self, node: FeedNode);
    /// Apply a mutation to one article in place
    fn update_article(&self, article_id: i64, apply: impl Fn(&mut Article));
    /// Count articles, optionally scoped to a category/feed/filter
    fn count_articles(
        &self,
        cat_id: Option<i64>,
        feed_id: Option<i64>,
        filter: Option<Filter>,
    ) -> usize;
    /// Find one article by id
    fn article_by_id(&self, id: i64) -> Option<Article>;
    /// Clone out the articles matching the scope/filter
    fn articles(
        &self,
        cat_id: Option<i64>,
        feed_id: Option<i64>,
        filter: Option<Filter>,
    ) -> Vec<Article>;
    /// Ordered article ids for Reader swipe-nav within one feed
    /// (`keep` stays in the list even if the filter would drop it)
    fn feed_nav_ids(&self, cat_id: i64, feed_id: i64, filter: Filter, keep: i64) -> Vec<i64>;
    /// Mark a list of articles as read
    fn mark_read(&self, ids: &[i64]);
}

impl Tree for GlobalSignal<Option<Vec<CategoryNode>>> {
    fn build(&self, cats: Vec<Category>, feeds: Vec<Feed>, articles: Vec<Article>) {
        let body = cats
            .into_iter()
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
            .collect();
        *self.write() = Some(body);
    }

    fn toggle_category(&self, node: CategoryNode) {
        if let Some(cats) = self.write().as_mut() {
            if let Some(cat) = cats.into_iter().find(|c| c.category.id == node.category.id) {
                tracing::debug!("toggle expanded category: {}", node.category.label);
                cat.expanded = !cat.expanded;
                return;
            }
        }
    }

    fn toggle_feed(&self, node: FeedNode) {
        if let Some(cats) = self.write().as_mut() {
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

    fn update_article(&self, article_id: i64, apply: impl Fn(&mut Article)) {
        if let Some(cats) = self.write().as_mut() {
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

    fn count_articles(
        &self,
        cat_id: Option<i64>,
        feed_id: Option<i64>,
        filter: Option<Filter>,
    ) -> usize {
        let tree = self.read();
        iter_articles(tree.as_deref().unwrap_or_default(), cat_id, feed_id, filter).count()
    }

    fn article_by_id(&self, id: i64) -> Option<Article> {
        let tree = self.read();
        iter_articles(tree.as_deref().unwrap_or_default(), None, None, None)
            .find(|a| a.id == id)
            .cloned()
    }

    fn articles(
        &self,
        cat_id: Option<i64>,
        feed_id: Option<i64>,
        filter: Option<Filter>,
    ) -> Vec<Article> {
        let tree = self.read();
        iter_articles(tree.as_deref().unwrap_or_default(), cat_id, feed_id, filter)
            .cloned()
            .collect()
    }

    fn feed_nav_ids(&self, cat_id: i64, feed_id: i64, filter: Filter, keep: i64) -> Vec<i64> {
        let tree = self.read();
        iter_articles(
            tree.as_deref().unwrap_or_default(),
            Some(cat_id),
            Some(feed_id),
            None,
        )
        .filter(|a| a.matches(filter) || a.id == keep)
        .map(|a| a.id)
        .collect()
    }

    fn mark_read(&self, ids: &[i64]) {
        if let Some(cats) = self.write().as_mut() {
            cats.iter_mut()
                .flat_map(|c| c.feeds.iter_mut().flatten())
                .flat_map(|f| f.articles.iter_mut().flatten())
                .filter(|a| ids.contains(&a.id))
                .for_each(|a| a.is_read = true);
        }
    }
}
