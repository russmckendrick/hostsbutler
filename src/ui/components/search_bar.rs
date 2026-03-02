use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::{App, AppMode};
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    if app.mode != AppMode::Search && app.search_query.is_empty() {
        return;
    }

    let style = if app.mode == AppMode::Search {
        Theme::search_highlight()
    } else {
        Theme::dim()
    };

    let cursor = if app.mode == AppMode::Search { "_" } else { "" };

    let line = Line::from(vec![
        Span::styled(" / ", style),
        Span::styled(&app.search_query, style),
        Span::styled(cursor, style),
    ]);

    let paragraph = Paragraph::new(line);
    f.render_widget(paragraph, area);
}
