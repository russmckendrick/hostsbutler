use ratatui::{
    Frame,
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

use crate::app::{App, AppMode};
use crate::ui::components;
use crate::ui::layout::AppLayout;
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, app: &App) {
    f.render_widget(Block::default().style(Theme::app()), f.area());

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
    let readonly = if app.readonly { " [Read-only]" } else { "" };
    let path = app.hosts.path.display().to_string();
    let title = format!(" HostsButler v{}", env!("CARGO_PKG_VERSION"));
    let right = truncate_left(
        &format!("{}{}{}", path, dirty, readonly),
        area.width.saturating_sub(title.len() as u16 + 1) as usize,
    );

    let line = Line::from(vec![
        Span::styled(title.clone(), Theme::header()),
        Span::styled(
            format!(
                "{}{}",
                " ".repeat(
                    area.width
                        .saturating_sub(title.len() as u16 + right.len() as u16)
                        as usize
                ),
                ""
            ),
            Theme::header(),
        ),
        Span::styled(right, Theme::header()),
    ]);

    f.render_widget(Paragraph::new(line).style(Theme::header()), area);
}

fn truncate_left(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    if text.len() <= max_width {
        return text.to_string();
    }

    if max_width <= 3 {
        return ".".repeat(max_width);
    }

    let keep = max_width - 3;
    let suffix = &text[text.len() - keep..];
    format!("...{}", suffix)
}
