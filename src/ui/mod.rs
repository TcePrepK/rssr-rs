mod chrome;
mod content;
mod editor;
mod popups;
mod settings;

use crate::app::App;
use crate::models::{AppState, Tab};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Color,
    symbols,
    Frame,
};

// ── Catppuccin Mocha palette ──────────────────────────────────────────────────
pub(crate) const MAUVE: Color = Color::Rgb(203, 166, 247);
pub(crate) const BLUE: Color = Color::Rgb(137, 180, 250);
pub(crate) const GREEN: Color = Color::Rgb(166, 227, 161);
pub(crate) const PEACH: Color = Color::Rgb(250, 179, 135);
pub(crate) const BASE: Color = Color::Rgb(30, 30, 46);
pub(crate) const MANTLE: Color = Color::Rgb(24, 24, 37);
pub(crate) const TEXT: Color = Color::Rgb(205, 214, 244);
pub(crate) const SUBTEXT0: Color = Color::Rgb(166, 173, 200);
pub(crate) const SURFACE0: Color = Color::Rgb(49, 50, 68);
pub(crate) const YELLOW: Color = Color::Rgb(249, 226, 175);
pub(crate) const TEAL: Color = Color::Rgb(148, 226, 213);
pub(crate) const SKY: Color = Color::Rgb(137, 220, 235);
pub(crate) const PINK: Color = Color::Rgb(245, 194, 231);
pub(crate) const RED: Color = Color::Rgb(243, 139, 168);

pub(crate) const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Returns the border set based on the user's rounded-border preference.
pub(crate) fn border_set(rounded: bool) -> symbols::border::Set<'static> {
    if rounded { symbols::border::ROUNDED } else { symbols::border::PLAIN }
}

/// Fixed palette for category headers (cycles by category id).
pub(crate) const CATEGORY_COLORS: &[Color] = &[MAUVE, BLUE, GREEN, PEACH, YELLOW, TEAL, SKY, PINK];

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    chrome::draw_tab_bar(f, app, chunks[0]);
    chrome::draw_footer(f, app, chunks[2]);

    match app.selected_tab {
        Tab::Feeds => content::draw_feeds_tab(f, app, chunks[1]),
        Tab::Favorites => content::draw_favorites_tab(f, app, chunks[1]),
        Tab::Settings => settings::draw_settings_tab(f, app, chunks[1]),
    }

    if app.state == AppState::AddFeed {
        popups::draw_add_feed_popup(f, app);
    }
    if matches!(app.state, AppState::OPMLExportPath | AppState::OPMLImportPath) {
        popups::draw_opml_path_popup(f, app);
    }
    if app.state == AppState::ClearData {
        popups::draw_confirm_delete_all(f, app);
    }
    if app.state == AppState::ClearArticleCache {
        popups::draw_confirm_clear_cache(f, app);
    }
    if let Some((cat_id, feed_count)) = app.editor_delete_cat {
        popups::draw_confirm_delete_cat(f, app, cat_id, feed_count);
    }
}
