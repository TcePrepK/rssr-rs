use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::CategoryId;

/// A single RSS/Atom feed source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feed {
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub category_id: Option<CategoryId>,
    #[serde(default)]
    pub order: usize,
    #[serde(skip, default)]
    pub unread_count: usize,
    #[serde(skip, default)]
    pub articles: Vec<Article>,
    #[serde(skip, default)]
    pub fetched: bool,
    #[serde(skip, default)]
    pub fetch_error: Option<String>,
    #[serde(skip)]
    pub feed_updated_secs: Option<i64>,
    #[serde(default)]
    pub last_fetched_secs: Option<i64>,
}

/// A single article entry from a feed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub title: String,
    pub description: String,
    pub link: String,
    pub is_read: bool,
    /// Whether this article has been saved to a category. Runtime flag; not persisted on Article.
    #[serde(default)]
    pub is_saved: bool,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub source_feed: String,
    #[serde(default)]
    pub published_secs: Option<i64>,
}

fn default_true() -> bool {
    true
}

/// A user-defined category for saved articles.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SavedCategory {
    pub id: u32,
    pub name: String,
}

/// An article saved by the user into a named category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedArticle {
    pub article: Article,
    pub category_id: u32,
}

/// User-specific persistent data.
#[derive(Serialize, Deserialize, Default)]
pub struct UserData {
    pub read_links: HashSet<String>,
    /// New: articles saved to user-defined categories.
    #[serde(default)]
    pub saved_articles: Vec<SavedArticle>,
    /// New: user-defined save categories.
    #[serde(default)]
    pub saved_categories: Vec<SavedCategory>,
    /// Legacy field: populated when reading old user_data.json. Migrated on load, never re-written.
    #[serde(default, skip_serializing)]
    pub starred_articles: Vec<Article>,
    #[serde(default)]
    pub save_article_content: bool,
    #[serde(default)]
    pub border_rounded: bool,
    #[serde(default = "default_true")]
    pub eager_article_fetch: bool,
    #[serde(default = "default_true")]
    pub auto_fetch_on_start: bool,
}
