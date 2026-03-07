use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::app::App;
use crate::ui::layout::centered_rect;
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let popup = centered_rect(70, 70, area);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Backup Manager ")
        .borders(Borders::ALL)
        .style(Theme::modal_bg())
        .border_style(Theme::border_focused());

    if app.backup_list.is_empty() {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled("  No backups found.", Theme::dim())),
            Line::from(""),
            Line::from(Span::styled(
                "  Press [c] to create a backup.",
                Theme::dim(),
            )),
        ];
        let inner = block.inner(popup);
        f.render_widget(block, popup);
        f.render_widget(ratatui::widgets::Paragraph::new(text), inner);
        return;
    }

    let items: Vec<ListItem> = app
        .backup_list
        .iter()
        .enumerate()
        .map(|(idx, backup)| {
            let style = if idx == app.selected_backup {
                Theme::selected()
            } else {
                Theme::normal()
            };

            let prefix = if idx == app.selected_backup {
                ">> "
            } else {
                "   "
            };

            let desc = backup.description.as_deref().unwrap_or("");

            let size_kb = backup.size_bytes / 1024;

            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(&backup.filename, style),
                Span::styled(format!("  ({}KB)", size_kb), Theme::dim()),
                Span::styled(format!("  {}", desc), Theme::dim()),
            ]))
        })
        .collect();

    let inner = block.inner(popup);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    f.render_widget(block, popup);
    let list = List::new(items);
    f.render_widget(list, chunks[0]);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  [Enter/r] ", Theme::search_highlight()),
            Span::styled("roll back", Theme::dim()),
            Span::styled("  [c] ", Theme::search_highlight()),
            Span::styled("create", Theme::dim()),
            Span::styled("  [d] ", Theme::search_highlight()),
            Span::styled("delete", Theme::dim()),
        ])),
        chunks[1],
    );
}
