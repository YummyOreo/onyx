use eyre::{eyre, Result};
use ratatui::prelude::*;

pub fn centered_rect(percent_x: u16, size_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length((r.height - size_y) / 2),
                Constraint::Length(size_y),
                Constraint::Length((r.height - size_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn get_file_color(kind: &std::fs::FileType) -> Result<Color> {
    if kind.is_dir() {
        return Ok(Color::Cyan);
    }
    if kind.is_file() {
        return Ok(Color::White);
    }
    if kind.is_symlink() {
        return Ok(Color::Green);
    }
    Err(eyre!("unreachable"))
}

pub fn convert_sytax_style(s_style: syntect::highlighting::Style) -> Style {
    let mut style = Style::default();
    style = style.fg(Color::Rgb(
        s_style.foreground.r,
        s_style.foreground.g,
        s_style.foreground.b,
    ));
    style
}
