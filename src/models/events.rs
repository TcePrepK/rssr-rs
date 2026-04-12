use super::{Article, FeedSource};

/// Events flowing over the MPSC channel into the main event loop.
#[derive(Debug)]
pub enum AppEvent {
    Input(crossterm::event::KeyEvent),
    Tick,
    /// (feed_idx, Result<(articles, xml_updated_secs), error>)
    FeedFetched(usize, Result<(Vec<Article>, Option<i64>), String>),
    FullArticleFetched(FeedSource, usize, Result<String, String>),
    /// Result of background feed-title fetch during AddFeed.
    FeedTitleFetched(Result<String, String>),
}
