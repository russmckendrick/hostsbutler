use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::AppMode;
use crate::ui::layout::centered_rect;
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, mode: &AppMode, area: Rect) {
    let (title, message, options) = match mode {
        AppMode::ConfirmDelete(_) => (
            " Confirm Delete ",
            "Are you sure you want to delete this entry?",
            "[y] Yes  [n/Esc] No",
        ),
        AppMode::ConfirmSave => (
            " Unsaved Changes ",
            "You have unsaved changes. Save before quitting?",
            "[y] Save & Quit  [n] Quit without saving  [Esc] Cancel",
        ),
        _ => return,
    };

    let popup = centered_rect(50, 20, area);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Theme::border_focused());

    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(format!("  {}", message), Theme::normal())),
        Line::from(""),
        Line::from(Span::styled(format!("  {}", options), Theme::dim())),
    ];

    f.render_widget(Paragraph::new(text), inner);
}
