use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    app::App,
    fetch::{fetch_feed, fetch_feed_title},
    models::{AddFeedStep, AppEvent, AppState, Feed, SettingsItem},
    storage::{
        article_cache_size, clear_all_data, clear_article_cache, default_export_path,
        expand_home_dir, export_opml_to_path, import_opml_from_path, save_categories, save_feeds,
        save_user_data,
    },
};

pub(super) fn handle_settings(app: &mut App, key: KeyEvent) -> bool {
    // Refresh cache size each time the user interacts with the settings screen.
    app.article_cache_size = article_cache_size();
    match key.code {
        KeyCode::Char('q') => return true,
        KeyCode::Esc => app.unselect(),
        KeyCode::Tab => app.switch_tab_right(),
        KeyCode::BackTab => app.switch_tab_left(),
        KeyCode::Up => app.previous(),
        KeyCode::Down => app.next(),
        KeyCode::Enter => match app.settings_selected {
            SettingsItem::ImportOpml => {
                app.opml_path_input.clear();
                app.state = AppState::OPMLImportPath;
            }
            SettingsItem::ExportOpml => {
                app.opml_path_input = default_export_path();
                app.state = AppState::OPMLExportPath;
            }
            SettingsItem::ClearData => {
                app.state = AppState::ClearData;
            }
            SettingsItem::SaveArticleContent => {
                app.user_data.save_article_content = !app.user_data.save_article_content;
                let _ = save_user_data(&app.user_data);
                let state = if app.user_data.save_article_content { "ON" } else { "OFF" };
                app.status_msg = format!("Save Article Content: {state}");
            }
            SettingsItem::ClearArticleCache => {
                app.state = AppState::ClearArticleCache;
            }
            SettingsItem::EagerArticleFetch => {
                app.user_data.eager_article_fetch = !app.user_data.eager_article_fetch;
                let _ = save_user_data(&app.user_data);
                let state = if app.user_data.eager_article_fetch { "ON" } else { "OFF" };
                app.status_msg = format!("Eager Article Fetch: {state}");
            }
            SettingsItem::BorderStyle => {
                app.user_data.border_rounded = !app.user_data.border_rounded;
                let _ = save_user_data(&app.user_data);
                let state = if app.user_data.border_rounded { "ON" } else { "OFF" };
                app.status_msg = format!("Rounded Borders: {state}");
            }
        },
        _ => {}
    }
    false
}

pub(super) fn handle_add_feed(app: &mut App, key: KeyEvent, tx: &UnboundedSender<AppEvent>) {
    if app.add_feed_step == AddFeedStep::Url {
        match key.code {
            KeyCode::Enter => {
                let url = app.input.trim().to_string();
                if url.is_empty() {
                    return;
                }
                app.add_feed_url = url.clone();
                app.input.clear();
                app.add_feed_fetched_title = None;
                app.add_feed_step = AddFeedStep::Title;
                let tx2 = tx.clone();
                tokio::spawn(async move {
                    let result = fetch_feed_title(&url).await;
                    let _ = tx2.send(AppEvent::FeedTitleFetched(result));
                });
            }
            KeyCode::Char(c) => app.input.push(c),
            KeyCode::Backspace => {
                app.input.pop();
            }
            KeyCode::Esc => app.unselect(),
            _ => {}
        }
    } else {
        match key.code {
            KeyCode::Enter => {
                let typed = app.input.trim().to_string();
                let title = if typed.is_empty() {
                    match app.add_feed_fetched_title.clone() {
                        Some(t) if !t.is_empty() => t,
                        _ => {
                            app.status_msg = "Title is required.".to_string();
                            return;
                        }
                    }
                } else {
                    typed
                };
                let url = app.add_feed_url.clone();
                let target_category = app.add_feed_target_category.take();
                let next_order = if let Some(insert_at) = app.add_feed_target_order.take() {
                    // Shift all sibling feeds with order >= insert_at up by 1 to make room.
                    for f in app.feeds.iter_mut() {
                        if f.category_id == target_category && f.order >= insert_at {
                            f.order += 1;
                        }
                    }
                    insert_at
                } else {
                    app.feeds.iter().map(|f| f.order).max().unwrap_or(0) + 1
                };
                app.feeds.push(Feed {
                    title: title.clone(),
                    url: url.clone(),
                    category_id: target_category,
                    order: next_order,
                    unread_count: 0,
                    articles: vec![],
                    fetched: false,
                    fetch_error: None,
                    feed_updated_secs: None,
                });
                let _ = save_feeds(&app.feeds);
                app.status_msg = format!("Feed '{title}' added!");
                let tx2 = tx.clone();
                let idx = app.feeds.len() - 1;
                tokio::spawn(async move {
                    let result = fetch_feed(&url).await;
                    let _ = tx2.send(AppEvent::FeedFetched(idx, result));
                });
                app.input.clear();
                app.add_feed_step = AddFeedStep::Url;
                app.add_feed_url.clear();
                app.add_feed_fetched_title = None;
                app.state = app.add_feed_return_state.clone();
            }
            KeyCode::Char(c) => app.input.push(c),
            KeyCode::Backspace => {
                app.input.pop();
            }
            KeyCode::Esc => app.unselect(),
            _ => {}
        }
    }
}

pub(super) fn handle_confirm_delete_all(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            app.feeds.clear();
            app.categories.clear();
            app.user_data = crate::models::UserData::default();
            app.favorite_view_articles.clear();
            app.selected_feed = 0;
            app.selected_article = 0;
            app.sidebar_cursor = 0;
            let _ = clear_all_data();
            app.status_msg = "All data cleared.".to_string();
            app.state = AppState::SettingsList;
        }
        KeyCode::Esc | KeyCode::Char('q') => app.state = AppState::SettingsList,
        _ => {}
    }
}

pub(super) fn handle_confirm_clear_cache(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            let _ = clear_article_cache();
            app.article_cache_size = 0;
            app.status_msg = "Article cache cleared.".to_string();
            app.state = AppState::SettingsList;
        }
        KeyCode::Esc | KeyCode::Char('q') => app.state = AppState::SettingsList,
        _ => {}
    }
}

pub(super) fn handle_opml_path(app: &mut App, key: KeyEvent, tx: &UnboundedSender<AppEvent>) {
    match key.code {
        KeyCode::Enter => {
            let raw = app.opml_path_input.trim().to_string();
            if raw.is_empty() {
                app.status_msg = "Path cannot be empty.".to_string();
                return;
            }
            let path = expand_home_dir(&raw);
            if app.state == AppState::OPMLExportPath {
                match export_opml_to_path(&path, &app.feeds, &app.categories) {
                    Ok(()) => app.status_msg = format!("Exported to {raw}"),
                    Err(e) => app.status_msg = format!("Export failed: {e}"),
                }
            } else {
                match import_opml_from_path(&path, &app.feeds, &app.categories) {
                    Ok((new_feeds, new_cats)) if new_feeds.is_empty() && new_cats.is_empty() => {
                        app.status_msg = "No new feeds found in OPML file.".to_string();
                    }
                    Ok((new_feeds, new_cats)) => {
                        let feed_count = new_feeds.len();
                        let cat_count = new_cats.len();
                        let first_new_idx = app.feeds.len();
                        for (i, feed) in new_feeds.iter().enumerate() {
                            let tx2 = tx.clone();
                            let url = feed.url.clone();
                            let idx = first_new_idx + i;
                            tokio::spawn(async move {
                                let result = fetch_feed(&url).await;
                                let _ = tx2.send(AppEvent::FeedFetched(idx, result));
                            });
                        }
                        app.feeds.extend(new_feeds);
                        app.categories.extend(new_cats);
                        let _ = save_feeds(&app.feeds);
                        let _ = save_categories(&app.categories);
                        app.status_msg =
                            format!("Imported {feed_count} feed(s), {cat_count} category(s)");
                    }
                    Err(e) => app.status_msg = format!("Import failed: {e}"),
                }
            }
            app.opml_path_input.clear();
            app.state = AppState::SettingsList;
        }
        KeyCode::Char(c) => app.opml_path_input.push(c),
        KeyCode::Backspace => {
            app.opml_path_input.pop();
        }
        KeyCode::Esc => app.unselect(),
        _ => {}
    }
}
