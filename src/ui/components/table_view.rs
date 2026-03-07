use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::app::{App, FocusPanel};
use crate::model::EntryStatus;
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.focus == FocusPanel::Table {
        Theme::border_focused()
    } else {
        Theme::border()
    };

    let header = Row::new(vec![
        Cell::from("Status"),
        Cell::from("IP Address"),
        Cell::from("Hostname(s)"),
        Cell::from("Group"),
        Cell::from("Comment"),
    ])
    .style(Theme::header());

    let rows: Vec<Row> = app
        .filtered_entry_ids
        .iter()
        .enumerate()
        .filter_map(|(idx, &id)| {
            let entry = app.hosts.find_entry(id)?;

            let status = match &entry.status {
                EntryStatus::Enabled => Span::styled("[*]", Theme::enabled()),
                EntryStatus::Disabled { .. } => Span::styled("[ ]", Theme::disabled()),
            };

            let row_style = if idx == app.selected_entry {
                Theme::selected()
            } else if !entry.status.is_enabled() {
                Theme::disabled()
            } else {
                Theme::normal()
            };

            Some(
                Row::new(vec![
                    Cell::from(Line::from(status)),
                    Cell::from(entry.ip.to_string()),
                    Cell::from(entry.hostnames.join(" ")),
                    Cell::from(entry.group.as_deref().unwrap_or("")),
                    Cell::from(entry.inline_comment.as_deref().unwrap_or("")),
                ])
                .style(row_style),
            )
        })
        .collect();

    let widths = [
        ratatui::layout::Constraint::Length(6),
        ratatui::layout::Constraint::Length(18),
        ratatui::layout::Constraint::Percentage(35),
        ratatui::layout::Constraint::Length(12),
        ratatui::layout::Constraint::Percentage(20),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Entries ")
                .style(Theme::surface())
                .border_style(border_style),
        )
        .row_highlight_style(Theme::selected())
        .highlight_symbol(">> ");

    f.render_widget(table, area);
}
