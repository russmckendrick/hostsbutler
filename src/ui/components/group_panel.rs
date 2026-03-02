use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use crate::app::{App, FocusPanel};
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.focus == FocusPanel::Groups {
        Theme::border_focused()
    } else {
        Theme::border()
    };

    let groups = app.hosts.groups();

    let items: Vec<ListItem> = app
        .groups_list
        .iter()
        .enumerate()
        .map(|(idx, name)| {
            let count = if name == "All" {
                app.hosts.entries().len()
            } else {
                groups
                    .iter()
                    .find(|g| g.name == *name)
                    .map(|g| g.entry_count)
                    .unwrap_or(0)
            };

            let style = if idx == app.selected_group {
                Theme::active_group()
            } else {
                Theme::normal()
            };

            let prefix = if idx == app.selected_group {
                "> "
            } else {
                "  "
            };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(format!("{} ({})", name, count), style),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Groups ")
            .border_style(border_style),
    );

    f.render_widget(list, area);
}
