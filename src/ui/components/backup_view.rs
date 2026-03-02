use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem},
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

    let list = List::new(items).block(block);
    f.render_widget(list, popup);
}
