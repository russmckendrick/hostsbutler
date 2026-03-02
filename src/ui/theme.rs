use ratatui::style::{Color, Modifier, Style};

pub struct Theme;

impl Theme {
    pub fn header() -> Style {
        Style::default().bg(Color::Blue).fg(Color::White)
    }

    pub fn selected() -> Style {
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }

    pub fn enabled() -> Style {
        Style::default().fg(Color::Green)
    }

    pub fn disabled() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    pub fn active_group() -> Style {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }

    pub fn error() -> Style {
        Style::default().fg(Color::Red)
    }

    pub fn success() -> Style {
        Style::default().fg(Color::Green)
    }

    pub fn search_highlight() -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }

    pub fn border() -> Style {
        Style::default().fg(Color::Gray)
    }

    pub fn border_focused() -> Style {
        Style::default().fg(Color::Cyan)
    }

    pub fn normal() -> Style {
        Style::default().fg(Color::White)
    }

    pub fn dim() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    pub fn modal_bg() -> Style {
        Style::default().bg(Color::Black)
    }

    pub fn field_active() -> Style {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    }

    pub fn field_inactive() -> Style {
        Style::default().fg(Color::Gray)
    }
}
