use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::{App, AppMode};
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    // Top line: search bar or shortcuts
    match app.mode {
        AppMode::Search => {
            super::search_bar::render(f, app, chunks[0]);
        }
        _ => {
            let shortcuts = match app.mode {
                AppMode::Normal => vec![
                    ("[/]", "Search"),
                    ("[Tab]", "Panels"),
                    ("[a]", "Add"),
                    ("[Enter]", "Edit"),
                    ("[Space]", "Toggle"),
                    ("[Ctrl+S]", "Save"),
                    ("[q]", "Quit"),
                    ("[?]", "Help"),
                ],
                AppMode::BackupManager => {
                    vec![
                        ("[Enter/r]", "Roll back"),
                        ("[c]", "Create"),
                        ("[d]", "Delete"),
                        ("[Esc]", "Close"),
                    ]
                }
                AppMode::ConfirmDelete(_) => {
                    vec![("[Enter/y]", "Delete"), ("[Esc/n]", "Cancel")]
                }
                AppMode::ConfirmSave => vec![
                    ("[Enter/y]", "Save & quit"),
                    ("[n]", "Quit without saving"),
                    ("[Esc]", "Cancel"),
                ],
                AppMode::Help => vec![("[Esc/?/q]", "Close")],
                AppMode::AddEntry | AppMode::EditEntry(_) => {
                    vec![("[Tab]", "Next"), ("[Enter]", "Save"), ("[Esc]", "Cancel")]
                }
                _ => vec![("[Esc]", "Cancel"), ("[Enter]", "Confirm")],
            };

            let spans: Vec<Span> = shortcuts
                .iter()
                .flat_map(|(key, desc)| {
                    vec![
                        Span::styled(format!(" {} ", key), Theme::search_highlight()),
                        Span::styled(format!("{} ", desc), Theme::dim()),
                    ]
                })
                .collect();

            let line = Line::from(spans);
            f.render_widget(Paragraph::new(line), chunks[0]);
        }
    }

    // Bottom line: toast or mode indicator
    if let Some(ref toast) = app.toast {
        super::toast::render(f, toast, chunks[1]);
    } else {
        let mode_text = match &app.mode {
            AppMode::Normal => "NORMAL",
            AppMode::Search => "SEARCH",
            AppMode::AddEntry => "ADD ENTRY",
            AppMode::EditEntry(_) => "EDIT ENTRY",
            AppMode::ConfirmDelete(_) => "CONFIRM DELETE",
            AppMode::ConfirmSave => "UNSAVED CHANGES",
            AppMode::BackupManager => "BACKUP MANAGER",
            AppMode::Help => "HELP",
        };

        let dirty = if app.hosts.dirty { " [Modified]" } else { "" };
        let readonly = if app.readonly { " [Read-only]" } else { "" };
        let entry_count = app.filtered_entry_ids.len();
        let total = app.hosts.entries().len();

        let line = Line::from(vec![
            Span::styled(format!(" {} ", mode_text), Theme::header()),
            Span::styled(
                format!("  {}/{} entries{}{}", entry_count, total, dirty, readonly),
                Theme::dim(),
            ),
        ]);

        f.render_widget(Paragraph::new(line), chunks[1]);
    }
}
