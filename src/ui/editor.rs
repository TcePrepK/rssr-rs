use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use ratatui::prelude::Stylize;

use crate::{
    app::{visible_tree_items, App},
    models::{AppState, FeedEditorMode, FeedTreeItem},
};

use super::{border_set, BASE, BLUE, CATEGORY_COLORS, GREEN, MANTLE, MAUVE, PEACH, SUBTEXT0, SURFACE0, TEXT, YELLOW};

pub(super) fn draw_feed_editor(f: &mut Frame, app: &App, area: Rect) {
    let is_rename = app.state == AppState::FeedEditorRename;
    let mode_label = match &app.editor_mode {
        FeedEditorMode::Normal => " NORMAL ",
        FeedEditorMode::Moving { .. } => " MOVE — navigate then Enter to drop, Esc to cancel ",
        FeedEditorMode::Renaming { .. } => " RENAME ",
        FeedEditorMode::NewCategory { .. } => " NEW CATEGORY ",
    };
    let mode_color = match &app.editor_mode {
        FeedEditorMode::Normal => BLUE,
        FeedEditorMode::Moving { .. } => YELLOW,
        _ => GREEN,
    };

    let block = Block::default()
        .border_set(border_set(app.user_data.border_rounded))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MAUVE))
        .bg(BASE)
        .title(Line::from(vec![
            Span::styled(
                " Feed Editor ",
                Style::default().fg(MAUVE).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                mode_label,
                Style::default().fg(mode_color).add_modifier(Modifier::BOLD),
            ),
        ]));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let tree = visible_tree_items(&app.categories, &app.feeds, &app.editor_collapsed);

    let moving_origin = match &app.editor_mode {
        FeedEditorMode::Moving { origin_render_idx } => Some(*origin_render_idx),
        _ => None,
    };

    let items: Vec<ListItem> = tree
        .iter()
        .enumerate()
        .map(|(render_idx, item)| {
            let selected = app.editor_cursor == render_idx;
            let is_ghost = moving_origin == Some(render_idx);
            let is_drop_target = moving_origin.is_some() && selected;

            match item {
                FeedTreeItem::Category { id, depth, collapsed } => {
                    let color = CATEGORY_COLORS[(id % CATEGORY_COLORS.len() as u64) as usize];
                    let indent = "  ".repeat(*depth as usize);
                    let icon = if *collapsed { "▶ " } else { "▼ " };
                    let cat_name = app
                        .categories
                        .iter()
                        .find(|c| c.id == *id)
                        .map(|c| c.name.as_str())
                        .unwrap_or("?");
                    let style = if is_drop_target {
                        Style::default().fg(MANTLE).bg(YELLOW).add_modifier(Modifier::BOLD)
                    } else if is_ghost {
                        Style::default().fg(SURFACE0)
                    } else if selected {
                        Style::default().fg(MANTLE).bg(color).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(color).add_modifier(Modifier::BOLD)
                    };
                    let drop_marker = if is_drop_target { "→ " } else { "" };
                    ListItem::new(Line::from(vec![
                        Span::raw(indent),
                        Span::styled(drop_marker, Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
                        Span::styled(icon, style),
                        Span::styled(cat_name, style),
                    ]))
                }
                FeedTreeItem::Feed { feeds_idx, depth } => {
                    let feed = &app.feeds[*feeds_idx];
                    let indent = "  ".repeat(*depth as usize);

                    // Inline rename input for the selected feed
                    if is_rename
                        && selected
                        && matches!(app.editor_mode, FeedEditorMode::Renaming { .. })
                    {
                        return ListItem::new(Line::from(vec![
                            Span::raw(indent),
                            Span::styled("  ✎ ", Style::default().fg(GREEN)),
                            Span::styled(app.editor_input.clone(), Style::default().fg(TEXT)),
                            Span::styled("█", Style::default().fg(GREEN)),
                        ]));
                    }

                    let style = if is_drop_target {
                        Style::default().fg(MANTLE).bg(YELLOW).add_modifier(Modifier::BOLD)
                    } else if is_ghost {
                        Style::default().fg(SURFACE0)
                    } else if selected {
                        Style::default().fg(MAUVE).bg(SURFACE0).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(TEXT)
                    };
                    let drop_marker = if is_drop_target { "→ " } else { "" };
                    ListItem::new(Line::from(vec![
                        Span::raw(indent),
                        Span::styled(drop_marker, Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
                        Span::styled("  ", Style::default().fg(PEACH)),
                        Span::styled(feed.title.clone(), style),
                        Span::styled(feed.unread_badge(), Style::default().fg(YELLOW)),
                    ]))
                }
            }
        })
        .collect();

    // Overlay new-category or category-rename input row at cursor
    let mut final_items = items;
    if let FeedEditorMode::NewCategory { parent_id } = &app.editor_mode
        && is_rename
    {
        let parent_id = *parent_id;
        // Determine depth and indent from parent category.
        let depth = if parent_id.is_some() {
            let parent_depth = tree.iter().find_map(|item| match item {
                FeedTreeItem::Category { id, depth, .. } if Some(*id) == parent_id => Some(*depth),
                _ => None,
            });
            parent_depth.map(|d| d + 1).unwrap_or(1)
        } else {
            0
        };
        let indent = "  ".repeat(depth as usize);
        let insert_at = app.editor_cursor.min(final_items.len());
        final_items.insert(
            insert_at,
            ListItem::new(Line::from(vec![
                Span::raw(indent),
                Span::styled("  ✎ ", Style::default().fg(GREEN)),
                Span::styled(app.editor_input.clone(), Style::default().fg(TEXT)),
                Span::styled("█", Style::default().fg(GREEN)),
            ])),
        );
    } else if is_rename && matches!(app.editor_mode, FeedEditorMode::Renaming { .. }) {
        let cursor = app.editor_cursor;
        if let Some(FeedTreeItem::Category { id, depth, .. }) = tree.get(cursor) {
            let indent = "  ".repeat(*depth as usize);
            let color = CATEGORY_COLORS[(id % CATEGORY_COLORS.len() as u64) as usize];
            final_items[cursor] = ListItem::new(Line::from(vec![
                Span::raw(indent),
                Span::styled("  ✎ ", Style::default().fg(color)),
                Span::styled(app.editor_input.clone(), Style::default().fg(TEXT)),
                Span::styled("█", Style::default().fg(color)),
            ]));
        }
    }

    if final_items.is_empty() {
        f.render_widget(
            Paragraph::new(
                " No feeds yet. Press [a] to add a feed or [n] to create a category.",
            )
            .style(Style::default().fg(SUBTEXT0)),
            inner,
        );
        return;
    }

    let mut state = ListState::default();
    state.select(Some(app.editor_cursor));
    f.render_stateful_widget(List::new(final_items), inner, &mut state);
}
