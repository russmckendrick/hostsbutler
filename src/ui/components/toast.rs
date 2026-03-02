use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::Toast;
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, toast: &Toast, area: Rect) {
    let style = if toast.is_error {
        Theme::error()
    } else {
        Theme::success()
    };

    let prefix = if toast.is_error { " ! " } else { " > " };

    let line = Line::from(vec![
        Span::styled(prefix, style),
        Span::styled(&toast.message, style),
    ]);

    f.render_widget(Paragraph::new(line), area);
}
