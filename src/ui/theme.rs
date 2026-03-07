use ratatui::style::{Color, Modifier, Style};

pub struct Theme;

impl Theme {
    const BASE03: Color = Color::Rgb(0, 43, 54);
    const BASE02: Color = Color::Rgb(7, 54, 66);
    const BASE01: Color = Color::Rgb(88, 110, 117);
    const BASE0: Color = Color::Rgb(131, 148, 150);
    const BASE1: Color = Color::Rgb(147, 161, 161);
    const YELLOW: Color = Color::Rgb(181, 137, 0);
    const RED: Color = Color::Rgb(220, 50, 47);
    const BLUE: Color = Color::Rgb(38, 139, 210);
    const CYAN: Color = Color::Rgb(42, 161, 152);
    const GREEN: Color = Color::Rgb(133, 153, 0);

    pub fn app() -> Style {
        Style::default().bg(Self::BASE03).fg(Self::BASE0)
    }

    pub fn surface() -> Style {
        Style::default().bg(Self::BASE03).fg(Self::BASE0)
    }

    pub fn header() -> Style {
        Style::default()
            .bg(Self::BASE02)
            .fg(Self::BASE1)
            .add_modifier(Modifier::BOLD)
    }

    pub fn selected() -> Style {
        Style::default()
            .bg(Self::BASE02)
            .fg(Self::YELLOW)
            .add_modifier(Modifier::BOLD)
    }

    pub fn enabled() -> Style {
        Style::default().fg(Self::GREEN)
    }

    pub fn disabled() -> Style {
        Style::default().fg(Self::BASE01)
    }

    pub fn active_group() -> Style {
        Style::default().fg(Self::CYAN).add_modifier(Modifier::BOLD)
    }

    pub fn error() -> Style {
        Style::default().fg(Self::RED)
    }

    pub fn success() -> Style {
        Style::default().fg(Self::GREEN)
    }

    pub fn search_highlight() -> Style {
        Style::default()
            .fg(Self::YELLOW)
            .add_modifier(Modifier::BOLD)
    }

    pub fn border() -> Style {
        Style::default().fg(Self::BASE01)
    }

    pub fn border_focused() -> Style {
        Style::default().fg(Self::BLUE)
    }

    pub fn normal() -> Style {
        Style::default().fg(Self::BASE0)
    }

    pub fn dim() -> Style {
        Style::default().fg(Self::BASE01)
    }

    pub fn modal_bg() -> Style {
        Style::default().bg(Self::BASE03).fg(Self::BASE0)
    }

    pub fn field_active() -> Style {
        Style::default().fg(Self::BASE1).bg(Self::BASE02)
    }

    pub fn field_inactive() -> Style {
        Style::default().fg(Self::BASE0)
    }
}
