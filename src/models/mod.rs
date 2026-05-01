//! Domain models: core types (Feed, Article), navigation states, events, and helper structs.

use serde::{Deserialize, Serialize};

mod core_types;
mod events;
pub mod feed;
mod navigation;
pub mod scroll;

pub use core_types::*;
pub use events::*;
pub use navigation::*;
pub use scroll::{ListScroll, TextScroll};

// ── Constants ─────────────────────────────────────────────────────────────────

/// URL used to identify the virtual Favorites feed (never persisted).
pub const FAVORITES_URL: &str = "internal:favorites";

/// Articles shorter than this are considered stubs and trigger a readability fetch.
pub const CONTENT_STUB_MAX_LEN: usize = 500;

// ── Category tree ─────────────────────────────────────────────────────────────

/// Stable identifier for a category node.
pub type CategoryId = u64;

/// A category node in the feed tree (persisted to categories.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    /// Unique identifier for this category.
    pub id: CategoryId,
    /// Display name of the category.
    pub name: String,
    /// None = root level. Some(id) = nested under that category.
    #[serde(default)]
    pub parent_id: Option<CategoryId>,
    /// Display order among siblings (lower = first).
    #[serde(default)]
    pub order: usize,
}

/// One visible row in the flattened category/feed tree.
#[derive(Debug, Clone)]
pub enum FeedTreeItem {
    /// Virtual entry always shown at the top of the sidebar — selects all articles across all feeds.
    AllFeeds,
    /// A category node with its tree depth and collapse state.
    Category {
        /// The category's unique ID.
        id: CategoryId,
        /// Indentation depth in the tree (0 = root).
        depth: u8,
        /// Whether this category's children are currently hidden.
        collapsed: bool,
    },
    /// A feed with its position in the flattened tree.
    Feed {
        /// Index into the app's global feeds list.
        feeds_idx: usize,
        /// Indentation depth in the tree (0 = root).
        depth: u8,
    },
}
