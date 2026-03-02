use ratatui::{
    Frame,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::{App, AppMode};
use crate::ui::components;
use crate::ui::layout::AppLayout;
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, app: &App) {
    let layout = AppLayout::new(f.area());

    // Title bar
    render_title_bar(f, app, layout.title_bar);

    // Group panel
    components::group_panel::render(f, app, layout.group_panel);

    // Entry table
    components::table_view::render(f, app, layout.entry_table);

    // Status bar
    components::status_bar::render(f, app, layout.status_bar);

    // Modal overlays
    match &app.mode {
        AppMode::AddEntry | AppMode::EditEntry(_) => {
            components::entry_dialog::render(f, app, f.area());
        }
        AppMode::ConfirmDelete(_) | AppMode::ConfirmSave => {
            components::confirm_dialog::render(f, &app.mode, f.area());
        }
        AppMode::BackupManager => {
            components::backup_view::render(f, app, f.area());
        }
        AppMode::Help => {
            components::help_overlay::render(f, f.area());
        }
        _ => {}
    }
}

fn render_title_bar(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let dirty = if app.hosts.dirty { " [Modified]" } else { "" };
    let path = app.hosts.path.display().to_string();

    let line = Line::from(vec![
        Span::styled(" HostsButler v0.1.0", Theme::header()),
        Span::styled(
            format!(
                "{}{}",
                " ".repeat(
                    area.width
                        .saturating_sub(20 + path.len() as u16 + dirty.len() as u16)
                        as usize
                ),
                ""
            ),
            Theme::header(),
        ),
        Span::styled(format!("{}{}  ", path, dirty), Theme::header()),
    ]);

    f.render_widget(Paragraph::new(line).style(Theme::header()), area);
}
