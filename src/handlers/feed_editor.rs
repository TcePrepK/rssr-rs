use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    app::{visible_tree_items, App},
    models::{AddFeedStep, AppEvent, AppState, Category, CategoryId, FeedEditorMode, FeedTreeItem},
    storage::{save_categories, save_feeds},
};

pub(super) fn handle_feed_editor(app: &mut App, key: KeyEvent, _tx: &UnboundedSender<AppEvent>) {
    match app.state {
        AppState::FeedEditorRename => match key.code {
            KeyCode::Enter => {
                let name = app.editor_input.trim().to_string();
                if !name.is_empty() {
                    match &app.editor_mode {
                        FeedEditorMode::NewCategory { parent_id } => {
                            let parent_id = *parent_id;
                            let next_id =
                                app.categories.iter().map(|c| c.id).max().unwrap_or(0) + 1;
                            let next_order = app
                                .categories
                                .iter()
                                .filter(|c| c.parent_id == parent_id)
                                .map(|c| c.order)
                                .max()
                                .unwrap_or(0)
                                + 1;
                            app.categories.push(Category {
                                id: next_id,
                                name,
                                parent_id,
                                order: next_order,
                            });
                            let _ = save_categories(&app.categories);
                        }
                        FeedEditorMode::Renaming { render_idx } => {
                            let items = visible_tree_items(
                                &app.categories,
                                &app.feeds,
                                &app.editor_collapsed,
                            );
                            match items.get(*render_idx) {
                                Some(FeedTreeItem::Category { id, .. }) => {
                                    if let Some(cat) =
                                        app.categories.iter_mut().find(|c| c.id == *id)
                                    {
                                        cat.name = name;
                                    }
                                    let _ = save_categories(&app.categories);
                                }
                                Some(FeedTreeItem::Feed { feeds_idx, .. }) => {
                                    if let Some(feed) = app.feeds.get_mut(*feeds_idx) {
                                        feed.title = name;
                                    }
                                    let _ = save_feeds(&app.feeds);
                                }
                                None => {}
                            }
                        }
                        _ => {}
                    }
                }
                app.editor_input.clear();
                app.editor_mode = FeedEditorMode::Normal;
                app.state = AppState::FeedEditor;
            }
            KeyCode::Esc => app.unselect(),
            KeyCode::Char(c) => app.editor_input.push(c),
            KeyCode::Backspace => {
                app.editor_input.pop();
            }
            _ => {}
        },
        AppState::FeedEditor => match &app.editor_mode.clone() {
            FeedEditorMode::Moving { origin_render_idx } => {
                let origin = *origin_render_idx;
                match key.code {
                    KeyCode::Char('j') | KeyCode::Down => app.next(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous(),
                    KeyCode::Enter => {
                        apply_move(app, origin);
                        app.editor_mode = FeedEditorMode::Normal;
                    }
                    KeyCode::Esc => {
                        app.editor_cursor = origin;
                        app.editor_mode = FeedEditorMode::Normal;
                    }
                    _ => {}
                }
            }
            FeedEditorMode::Normal => match key.code {
                KeyCode::Char('j') | KeyCode::Down => app.next(),
                KeyCode::Char('k') | KeyCode::Up => app.previous(),
                KeyCode::Enter => {
                    let items =
                        visible_tree_items(&app.categories, &app.feeds, &app.editor_collapsed);
                    match items.get(app.editor_cursor) {
                        Some(FeedTreeItem::Category { id, .. }) => {
                            let id = *id;
                            if app.editor_collapsed.contains(&id) {
                                app.editor_collapsed.remove(&id);
                            } else {
                                app.editor_collapsed.insert(id);
                            }
                        }
                        Some(FeedTreeItem::Feed { .. }) => {
                            app.editor_mode = FeedEditorMode::Moving {
                                origin_render_idx: app.editor_cursor,
                            };
                        }
                        None => {}
                    }
                }
                KeyCode::Char('a') => {
                    let items =
                        visible_tree_items(&app.categories, &app.feeds, &app.editor_collapsed);
                    let cursor_item = items.get(app.editor_cursor);
                    app.add_feed_target_category = match cursor_item {
                        Some(FeedTreeItem::Category { id, .. }) => Some(*id),
                        Some(FeedTreeItem::Feed { feeds_idx, .. }) => {
                            app.feeds.get(*feeds_idx).and_then(|f| f.category_id)
                        }
                        None => None,
                    };
                    // Determine insert order: after the cursor feed, or at start of cursor category.
                    app.add_feed_target_order = match cursor_item {
                        Some(FeedTreeItem::Feed { feeds_idx, .. }) => {
                            app.feeds.get(*feeds_idx).map(|f| f.order + 1)
                        }
                        Some(FeedTreeItem::Category { id, .. }) => {
                            let cat_id = *id;
                            let min = app
                                .feeds
                                .iter()
                                .filter(|f| f.category_id == Some(cat_id))
                                .map(|f| f.order)
                                .min();
                            Some(min.unwrap_or(0))
                        }
                        None => None,
                    };
                    app.input.clear();
                    app.add_feed_step = AddFeedStep::Url;
                    app.add_feed_url.clear();
                    app.add_feed_fetched_title = None;
                    app.add_feed_return_state = AppState::FeedEditor;
                    app.state = AppState::AddFeed;
                }
                KeyCode::Char('n') => {
                    let items =
                        visible_tree_items(&app.categories, &app.feeds, &app.editor_collapsed);
                    let parent_id = match items.get(app.editor_cursor) {
                        Some(FeedTreeItem::Category { id, .. }) => Some(*id),
                        Some(FeedTreeItem::Feed { feeds_idx, .. }) => {
                            app.feeds.get(*feeds_idx).and_then(|f| f.category_id)
                        }
                        None => None,
                    };
                    app.editor_input.clear();
                    app.editor_mode = FeedEditorMode::NewCategory { parent_id };
                    app.state = AppState::FeedEditorRename;
                }
                KeyCode::Char('r') => {
                    let items =
                        visible_tree_items(&app.categories, &app.feeds, &app.editor_collapsed);
                    let current_name = match items.get(app.editor_cursor) {
                        Some(FeedTreeItem::Category { id, .. }) => app
                            .categories
                            .iter()
                            .find(|c| c.id == *id)
                            .map(|c| c.name.clone())
                            .unwrap_or_default(),
                        Some(FeedTreeItem::Feed { feeds_idx, .. }) => app
                            .feeds
                            .get(*feeds_idx)
                            .map(|f| f.title.clone())
                            .unwrap_or_default(),
                        None => String::new(),
                    };
                    app.editor_input = current_name;
                    app.editor_mode = FeedEditorMode::Renaming {
                        render_idx: app.editor_cursor,
                    };
                    app.state = AppState::FeedEditorRename;
                }
                KeyCode::Char('d') => delete_at_cursor(app),
                KeyCode::Char('m') => {
                    app.editor_mode = FeedEditorMode::Moving {
                        origin_render_idx: app.editor_cursor,
                    };
                }
                KeyCode::Esc | KeyCode::Char('q') => app.unselect(),
                _ => {}
            },
            _ => {}
        },
        _ => {}
    }
}

fn delete_at_cursor(app: &mut App) {
    let items = visible_tree_items(&app.categories, &app.feeds, &app.editor_collapsed);
    match items.get(app.editor_cursor) {
        Some(FeedTreeItem::Category { id, .. }) => {
            let id = *id;
            orphan_category_feeds(app, id);
            remove_category_recursive(app, id);
            let _ = save_categories(&app.categories);
            let _ = save_feeds(&app.feeds);
            let new_len =
                visible_tree_items(&app.categories, &app.feeds, &app.editor_collapsed).len();
            if app.editor_cursor >= new_len && new_len > 0 {
                app.editor_cursor = new_len - 1;
            }
        }
        Some(FeedTreeItem::Feed { feeds_idx, .. }) => {
            let idx = *feeds_idx;
            if idx > 0 {
                app.feeds.remove(idx);
                let _ = save_feeds(&app.feeds);
                let new_len =
                    visible_tree_items(&app.categories, &app.feeds, &app.editor_collapsed).len();
                if app.editor_cursor >= new_len && new_len > 0 {
                    app.editor_cursor = new_len - 1;
                }
            }
        }
        None => {}
    }
}

fn orphan_category_feeds(app: &mut App, cat_id: CategoryId) {
    let children: Vec<CategoryId> = app
        .categories
        .iter()
        .filter(|c| c.parent_id == Some(cat_id))
        .map(|c| c.id)
        .collect();
    for child_id in children {
        orphan_category_feeds(app, child_id);
    }
    for feed in app.feeds.iter_mut() {
        if feed.category_id == Some(cat_id) {
            feed.category_id = None;
        }
    }
}

fn remove_category_recursive(app: &mut App, cat_id: CategoryId) {
    let children: Vec<CategoryId> = app
        .categories
        .iter()
        .filter(|c| c.parent_id == Some(cat_id))
        .map(|c| c.id)
        .collect();
    for child_id in children {
        remove_category_recursive(app, child_id);
    }
    app.categories.retain(|c| c.id != cat_id);
}

fn apply_move(app: &mut App, origin: usize) {
    let dest = app.editor_cursor;
    if origin == dest {
        return;
    }

    let items = visible_tree_items(&app.categories, &app.feeds, &app.editor_collapsed);
    let Some(src_item) = items.get(origin) else {
        return;
    };
    let dest_item = items.get(dest);

    let new_parent_cat: Option<CategoryId> = match dest_item {
        Some(FeedTreeItem::Category { id, .. }) => Some(*id),
        Some(FeedTreeItem::Feed { feeds_idx, .. }) => {
            app.feeds.get(*feeds_idx).and_then(|f| f.category_id)
        }
        None => None,
    };

    let dest_order = match dest_item {
        Some(FeedTreeItem::Feed { feeds_idx, .. }) => {
            app.feeds.get(*feeds_idx).map(|f| f.order).unwrap_or(0)
        }
        Some(FeedTreeItem::Category { id, .. }) => app
            .categories
            .iter()
            .find(|c| c.id == *id)
            .map(|c| c.order)
            .unwrap_or(0),
        None => usize::MAX,
    };

    match src_item.clone() {
        FeedTreeItem::Feed { feeds_idx, .. } => {
            if let Some(feed) = app.feeds.get_mut(feeds_idx) {
                feed.category_id = new_parent_cat;
                feed.order = dest_order;
            }
            let _ = save_feeds(&app.feeds);
        }
        FeedTreeItem::Category { id, .. } => {
            if let Some(cat) = app.categories.iter_mut().find(|c| c.id == id) {
                cat.parent_id = new_parent_cat;
                cat.order = dest_order;
            }
            let _ = save_categories(&app.categories);
        }
    }
}
