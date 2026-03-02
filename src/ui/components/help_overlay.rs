use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::ui::layout::centered_rect;
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, area: Rect) {
    let popup = centered_rect(60, 80, area);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Keyboard Shortcuts ")
        .borders(Borders::ALL)
        .border_style(Theme::border_focused());

    let shortcuts = vec![
        (
            "Navigation",
            vec![
                ("j / Down", "Move selection down"),
                ("k / Up", "Move selection up"),
                ("g / Home", "Jump to first entry"),
                ("G / End", "Jump to last entry"),
                ("Tab", "Switch focus: groups / table"),
            ],
        ),
        (
            "Entry Actions",
            vec![
                ("Space", "Toggle enable/disable"),
                ("a", "Add new entry"),
                ("e / Enter", "Edit selected entry"),
                ("d", "Delete (with confirmation)"),
            ],
        ),
        (
            "Search",
            vec![
                ("/", "Enter search mode"),
                ("Esc", "Exit search, clear filter"),
            ],
        ),
        (
            "File Operations",
            vec![
                ("Ctrl+S", "Save file"),
                ("Ctrl+R", "Reload from disk"),
                ("Ctrl+Z", "Undo"),
                ("Ctrl+Y", "Redo"),
            ],
        ),
        (
            "Other",
            vec![
                ("b", "Open backup manager"),
                ("t", "Test DNS resolution"),
                ("?", "Show/hide this help"),
                ("q", "Quit"),
            ],
        ),
    ];

    let mut lines = vec![Line::from("")];

    for (section, keys) in shortcuts {
        lines.push(Line::from(Span::styled(
            format!("  {}:", section),
            Theme::header(),
        )));

        for (key, desc) in keys {
            lines.push(Line::from(vec![
                Span::styled(format!("    {:14}", key), Theme::search_highlight()),
                Span::styled(desc, Theme::normal()),
            ]));
        }

        lines.push(Line::from(""));
    }

    lines.push(Line::from(Span::styled(
        "  Press Esc or ? to close",
        Theme::dim(),
    )));

    let inner = block.inner(popup);
    f.render_widget(block, popup);
    f.render_widget(Paragraph::new(lines), inner);
}
