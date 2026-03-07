use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::{App, AppMode};
use crate::ui::layout::centered_rect;
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let title = match app.mode {
        AppMode::AddEntry => " Add New Host Entry ",
        AppMode::EditEntry(_) => " Edit Host Entry ",
        _ => return,
    };

    let popup = centered_rect(50, 60, area);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Theme::modal_bg())
        .border_style(Theme::border_focused());

    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let fields = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // IP
            Constraint::Length(2), // Hostnames
            Constraint::Length(2), // Group
            Constraint::Length(2), // Comment
            Constraint::Length(2), // Enabled
            Constraint::Length(1), // Spacer
            Constraint::Length(2), // Error
            Constraint::Min(1),    // Help
        ])
        .split(inner);

    render_field(
        f,
        "IP Address:",
        &app.form.ip,
        app.form.active_field == 0,
        fields[0],
    );
    render_field(
        f,
        "Hostnames:",
        &app.form.hostnames,
        app.form.active_field == 1,
        fields[1],
    );
    render_field(
        f,
        "Group:",
        &app.form.group,
        app.form.active_field == 2,
        fields[2],
    );
    render_field(
        f,
        "Comment:",
        &app.form.comment,
        app.form.active_field == 3,
        fields[3],
    );

    // Enabled toggle
    let enabled_style = if app.form.active_field == 4 {
        Theme::field_active()
    } else {
        Theme::field_inactive()
    };
    let checkbox = if app.form.enabled { "[x]" } else { "[ ]" };
    let line = Line::from(vec![
        Span::styled("  Enabled:    ", Theme::normal()),
        Span::styled(checkbox, enabled_style),
    ]);
    f.render_widget(Paragraph::new(line), fields[4]);

    // Error message
    if let Some(ref error) = app.form.error {
        let error_line = Line::from(Span::styled(format!("  Error: {}", error), Theme::error()));
        f.render_widget(Paragraph::new(error_line), fields[6]);
    }

    // Help
    let help = Line::from(vec![
        Span::styled("  [Tab]", Theme::search_highlight()),
        Span::styled(" Next  ", Theme::dim()),
        Span::styled("[Enter]", Theme::search_highlight()),
        Span::styled(" Save  ", Theme::dim()),
        Span::styled("[Esc]", Theme::search_highlight()),
        Span::styled(" Cancel", Theme::dim()),
    ]);
    f.render_widget(Paragraph::new(help), fields[7]);
}

fn render_field(f: &mut Frame, label: &str, value: &str, active: bool, area: Rect) {
    let style = if active {
        Theme::field_active()
    } else {
        Theme::field_inactive()
    };

    let cursor = if active { "_" } else { "" };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    let label_line = Line::from(Span::styled(format!("  {}", label), Theme::normal()));
    f.render_widget(Paragraph::new(label_line), chunks[0]);

    let value_line = Line::from(vec![
        Span::styled("  [", Theme::dim()),
        Span::styled(value, style),
        Span::styled(cursor, style),
        Span::styled("]", Theme::dim()),
    ]);
    f.render_widget(Paragraph::new(value_line), chunks[1]);
}
