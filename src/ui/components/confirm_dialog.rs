use ratatui::{
    Frame,
    layout::Rect,
    layout::{Constraint, Layout},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::app::AppMode;
use crate::ui::layout::centered_rect;
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, mode: &AppMode, area: Rect) {
    let (title, message, options) = match mode {
        AppMode::ConfirmDelete(_) => (
            " Confirm Delete ",
            "Are you sure you want to delete this entry?",
            "[Enter/y] Delete  [Esc/n] Cancel",
        ),
        AppMode::ConfirmSave => (
            " Unsaved Changes ",
            "You have unsaved changes. Save before quitting?",
            "[Enter/y] Save & Quit  [n] Quit without saving  [Esc] Cancel",
        ),
        _ => return,
    };

    let popup = centered_rect(68, 24, area);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Theme::modal_bg())
        .border_style(Theme::border_focused());

    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let sections = Layout::default()
        .constraints([
            Constraint::Length(1),
            Constraint::Min(2),
            Constraint::Length(2),
        ])
        .split(inner);

    f.render_widget(
        Paragraph::new(vec![Line::from(Span::styled(
            format!("  {}", message),
            Theme::normal(),
        ))])
        .wrap(Wrap { trim: false }),
        sections[1],
    );
    f.render_widget(
        Paragraph::new(vec![Line::from(Span::styled(
            format!("  {}", options),
            Theme::dim(),
        ))])
        .wrap(Wrap { trim: false }),
        sections[2],
    );
}
