use std::fs;

use ratatui::{
    prelude::{Backend, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use syntect::{
    easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet, util::LinesWithEndings,
};

use crate::{filesystem::read::File, state::State};

use super::utils::{self, convert_sytax_style};

pub fn draw_side_panel(f: &mut Frame<'_, impl Backend>, chunk: Rect, state: &State) {
    let border = Block::default().borders(Borders::LEFT);
    let text = match state.files.get(state.selected) {
        Some(file) if file.is_dir().unwrap() => render_dir(file, &chunk),
        Some(file) if file.is_file().unwrap() => render_files(file),
        Some(file) if file.file_type.is_symlink() => render_symlink(file),
        Some(_) => vec![Line::from(Span::styled(
            "Unknown",
            Style::default().fg(Color::Gray),
        ))],
        None => vec![Line::from(Span::styled(
            "Empty",
            Style::default().fg(Color::Gray),
        ))],
    };
    let p = Paragraph::new(text).block(border);
    f.render_widget(p, chunk)
}

fn render_dir<'a>(file: &'a File, chunk: &'a Rect) -> Vec<Line<'a>> {
    let files = fs::read_dir(&file.path).unwrap().enumerate();

    let mut lines: Vec<Line> = vec![];
    for (i, file) in files {
        if i > chunk.height as usize + 1_usize {
            break;
        }
        let file = file.unwrap();
        lines.push(Line::from(Span::styled(
            file.file_name().to_string_lossy().to_string(),
            Style::default().fg(utils::get_file_color(&file.file_type().unwrap()).unwrap()),
        )))
    }
    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "Empty",
            Style::default().fg(Color::Gray),
        )))
    }
    lines
}

fn render_files(file: &File) -> Vec<Line> {
    let ps = SyntaxSet::load_defaults_newlines();
    let syntax = ps
        .find_syntax_by_extension(file.path.extension().unwrap_or_default().to_str().unwrap())
        .unwrap_or(ps.find_syntax_plain_text());

    let theme = ThemeSet::load_defaults().themes["Solarized (dark)"].clone();
    let mut h = HighlightLines::new(syntax, &theme);
    let content = String::from_utf8(fs::read(&file.path).unwrap()).unwrap_or("Binary".to_string());
    let mut lines: Vec<Line> = vec![];

    for line in LinesWithEndings::from(&content) {
        let ranges: Vec<(syntect::highlighting::Style, &str)> =
            h.highlight_line(line, &ps).unwrap();
        let mut line = vec![];
        for (style, s) in ranges {
            line.push(Span::styled(s.to_string(), convert_sytax_style(style)));
        }
        lines.push(Line::from(line));
    }
    lines
}

fn render_symlink(file: &File) -> Vec<Line> {
    let path = file
        .path
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .to_string();
    vec![Line::from(Span::styled(
        path,
        Style::default().fg(Color::LightBlue),
    ))]
}
