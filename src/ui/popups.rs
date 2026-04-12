use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use ratatui::prelude::Stylize;

use crate::{app::App, models::{AddFeedStep, AppState}};

use super::{border_set, BASE, BLUE, GREEN, MAUVE, SUBTEXT0, TEXT};

pub(super) fn draw_add_feed_popup(f: &mut Frame, app: &App) {
    let area = f.area();

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Percentage(35),
        ])
        .split(area);

    let center = |row: Rect| {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(row)[1]
    };

    let url_area = center(vertical[1]);
    let title_area = center(vertical[3]);

    // URL field
    f.render_widget(Clear, url_area);
    let url_content = if app.add_feed_step == AddFeedStep::Url {
        app.input.clone()
    } else {
        app.add_feed_url.clone()
    };
    let url_block = Block::default()
        .border_set(border_set(app.user_data.border_rounded))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if app.add_feed_step == AddFeedStep::Url {
            MAUVE
        } else {
            SUBTEXT0
        }))
        .bg(BASE)
        .title(Span::styled(
            " Feed URL ",
            Style::default().fg(BLUE).add_modifier(Modifier::BOLD),
        ));
    f.render_widget(
        Paragraph::new(url_content).block(url_block).style(Style::default().fg(TEXT)),
        url_area,
    );

    // Title field
    f.render_widget(Clear, title_area);
    let title_label = if app.add_feed_step == AddFeedStep::Url {
        " Feed Title (enter URL first) "
    } else {
        " Feed Title (Enter to save, Esc to cancel) "
    };
    let title_block = Block::default()
        .border_set(border_set(app.user_data.border_rounded))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if app.add_feed_step == AddFeedStep::Title {
            MAUVE
        } else {
            SUBTEXT0
        }))
        .bg(BASE)
        .title(Span::styled(
            title_label,
            Style::default().fg(BLUE).add_modifier(Modifier::BOLD),
        ));

    if app.add_feed_step == AddFeedStep::Title && app.input.is_empty() {
        match &app.add_feed_fetched_title {
            Some(t) if !t.is_empty() => {
                f.render_widget(
                    Paragraph::new(t.as_str())
                        .block(title_block)
                        .style(Style::default().fg(SUBTEXT0)),
                    title_area,
                );
                return;
            }
            Some(_) => {
                f.render_widget(
                    Paragraph::new("").block(title_block).style(Style::default().fg(TEXT)),
                    title_area,
                );
            }
            None => {
                f.render_widget(
                    Paragraph::new("⏳ Fetching title...")
                        .block(title_block)
                        .style(Style::default().fg(TEXT)),
                    title_area,
                );
            }
        }
        return;
    }

    let title_content = if app.add_feed_step == AddFeedStep::Title {
        app.input.clone()
    } else {
        String::new()
    };
    f.render_widget(
        Paragraph::new(title_content).block(title_block).style(Style::default().fg(TEXT)),
        title_area,
    );
}

pub(super) fn draw_confirm_delete_all(f: &mut Frame, app: &App) {
    let area = f.area();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(38),
            Constraint::Length(7),
            Constraint::Percentage(38),
        ])
        .split(area);
    let center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(vertical[1])[1];

    f.render_widget(Clear, center);
    let red = Color::Rgb(243, 139, 168); // Catppuccin Red — intentional, no constant
    let block = Block::default()
        .border_set(border_set(app.user_data.border_rounded))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(red))
        .bg(BASE)
        .title(Span::styled(
            " ⚠  Remove All Feeds ",
            Style::default().fg(red).add_modifier(Modifier::BOLD),
        ));
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  This will delete all feeds permanently.",
            Style::default().fg(TEXT),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [Enter] ", Style::default().fg(red).add_modifier(Modifier::BOLD)),
            Span::styled("Confirm   ", Style::default().fg(TEXT)),
            Span::styled("[Esc] ", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("Cancel", Style::default().fg(TEXT)),
        ]),
    ];
    f.render_widget(Paragraph::new(text).block(block), center);
}

pub(super) fn draw_confirm_clear_cache(f: &mut Frame, app: &App) {
    use super::RED;
    let area = f.area();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(38),
            Constraint::Length(7),
            Constraint::Percentage(38),
        ])
        .split(area);
    let center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(vertical[1])[1];

    f.render_widget(Clear, center);
    let block = Block::default()
        .border_set(border_set(app.user_data.border_rounded))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(RED))
        .bg(BASE)
        .title(Span::styled(
            " ⚠  Clear Article Cache ",
            Style::default().fg(RED).add_modifier(Modifier::BOLD),
        ));
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  This will delete all saved article content.",
            Style::default().fg(super::TEXT),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [Enter] ", Style::default().fg(RED).add_modifier(Modifier::BOLD)),
            Span::styled("Confirm   ", Style::default().fg(super::TEXT)),
            Span::styled("[Esc] ", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("Cancel", Style::default().fg(super::TEXT)),
        ]),
    ];
    f.render_widget(Paragraph::new(text).block(block), center);
}

pub(super) fn draw_opml_path_popup(f: &mut Frame, app: &App) {
    let is_export = app.state == AppState::OPMLExportPath;
    let title = if is_export {
        " Export OPML — destination path "
    } else {
        " Import OPML — source path "
    };

    let area = f.area();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(3),
            Constraint::Percentage(40),
        ])
        .split(area);

    let center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Percentage(70),
            Constraint::Percentage(15),
        ])
        .split(vertical[1])[1];

    f.render_widget(Clear, center);
    let block = Block::default()
        .border_set(border_set(app.user_data.border_rounded))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MAUVE))
        .bg(BASE)
        .title(Span::styled(
            title,
            Style::default().fg(BLUE).add_modifier(Modifier::BOLD),
        ));

    f.render_widget(
        Paragraph::new(app.opml_path_input.clone())
            .block(block)
            .style(Style::default().fg(TEXT)),
        center,
    );
}
