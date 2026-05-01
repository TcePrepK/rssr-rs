//! Key handlers for the main feed list and saved articles category list views.
//!
//! Manages navigation, feed refresh, feed editor entry, and category collapse/expand.

use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    app::{App, sidebar_tree_items},
    fetch::fetch_feed,
    models::{AppEvent, AppState, FeedEditorMode, FeedTreeItem},
};

/// Handle key input while viewing the main feed list; supports navigation, refresh, and feed editor entry.
pub(super) fn handle_feed_list(
    app: &mut App,
    key: KeyEvent,
    tx: &UnboundedSender<AppEvent>,
) -> bool {
    match key.code {
        KeyCode::Char('q') => return true,
        KeyCode::Char('r') if !app.in_saved_context => {
            let idx = app.selected_feed;
            if let Some(feed) = app.feeds.get_mut(idx) {
                let url = feed.url.clone();
                let title = feed.title.clone();
                feed.fetched = false;
                feed.fetch_error = None;
                app.set_status(format!("Refreshing {title}..."));
                app.feeds_pending += 1;
                app.feeds_total += 1;
                let tx2 = tx.clone();
                tokio::spawn(async move {
                    let result = fetch_feed(&url).await;
                    let _ = tx2.send(AppEvent::FeedFetched(idx, result));
                });
            }
        }
        KeyCode::Char('R') if !app.in_saved_context => {
            let count = app
                .feeds
                .iter()
                .filter(|f| f.url != crate::models::FAVORITES_URL)
                .count();
            if count > 0 {
                app.feeds_total += count;
                app.feeds_pending += count;
                app.set_status("Fetching all feeds...");
                for (idx, feed) in app.feeds.iter_mut().enumerate() {
                    if feed.url == crate::models::FAVORITES_URL {
                        continue;
                    }
                    feed.fetched = false;
                    feed.fetch_error = None;
                    let url = feed.url.clone();
                    let tx2 = tx.clone();
                    tokio::spawn(async move {
                        let result = fetch_feed(&url).await;
                        let _ = tx2.send(AppEvent::FeedFetched(idx, result));
                    });
                }
            }
        }
        KeyCode::Down => app.next(),
        KeyCode::Up => app.previous(),
        KeyCode::Tab => app.switch_tab_right(),
        KeyCode::BackTab => app.switch_tab_left(),
        KeyCode::Enter => app.select(),
        KeyCode::Char('e') => {
            app.editor_cursor = 0;
            app.editor_collapsed = app.sidebar_collapsed.clone();
            app.editor_mode = FeedEditorMode::Normal;
            app.state = AppState::FeedEditor;
        }
        KeyCode::Char('g') => {
            // Jump to top of feed list
            app.sidebar_cursor = 0;
            app.sidebar_title_start_tick = app.tick;

            let items = sidebar_tree_items(&app.categories, &app.feeds, &app.sidebar_collapsed);
            match items.first() {
                Some(FeedTreeItem::Feed { feeds_idx, .. }) => {
                    app.selected_feed = *feeds_idx;
                }
                Some(FeedTreeItem::AllFeeds) => {
                    app.populate_all_feeds_view();
                }
                _ => {}
            }
        }
        KeyCode::Char('C') => {
            // Toggle collapse/expand all categories
            let all_collapsed = app
                .categories
                .iter()
                .all(|c| app.sidebar_collapsed.contains(&c.id));
            if all_collapsed {
                app.sidebar_collapsed.clear();
            } else {
                for category in &app.categories {
                    app.sidebar_collapsed.insert(category.id);
                }
            }
            app.sidebar_cursor = 0;
            app.sidebar_title_start_tick = app.tick;
            let items = sidebar_tree_items(&app.categories, &app.feeds, &app.sidebar_collapsed);
            match items.first() {
                Some(FeedTreeItem::Feed { feeds_idx, .. }) => {
                    app.selected_feed = *feeds_idx;
                }
                Some(FeedTreeItem::AllFeeds) => {
                    app.populate_all_feeds_view();
                }
                _ => {}
            }
        }
        KeyCode::Char(' ') => {
            let items = sidebar_tree_items(&app.categories, &app.feeds, &app.sidebar_collapsed);
            if let Some(FeedTreeItem::Category { id, .. }) = items.get(app.sidebar_cursor) {
                app.toggle_category_collapse(*id);
            }
        }
        _ => {}
    }
    false
}

/// Handle key input while viewing the saved articles category list.
pub(super) fn handle_saved_feed_list(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('q') => return true,
        KeyCode::Tab => app.switch_tab_right(),
        KeyCode::BackTab => app.switch_tab_left(),
        KeyCode::Char('j') | KeyCode::Down => app.next(),
        KeyCode::Char('k') | KeyCode::Up => app.previous(),
        KeyCode::Enter => app.select(),
        KeyCode::Char('e') => {
            app.saved_cat_editor_scroll.set(0);
            app.state = AppState::SavedCategoryEditor;
        }
        KeyCode::Esc => app.unselect(),
        _ => {}
    }
    false
}
