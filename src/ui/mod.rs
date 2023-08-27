use std::{fs, io};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use eyre::{eyre, Context, ContextCompat, Result};
use ratatui::{
    prelude::{Backend, Constraint, CrosstermBackend, Direction, Layout, Rect},
    style::{Color, Style},
    text::{self, Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use syntect::{
    easy::{HighlightFile, HighlightLines},
    highlighting::ThemeSet,
    parsing::SyntaxSet,
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
};

use crate::{
    filesystem::{read::File, utils::FileType},
    main,
    state::InfoKind,
    Mode, State,
};

use self::utils::convert_sytax_style;

pub mod input;
mod utils;

const UI_ERROR_WRAP: &str = "Error while rendering ui:";

pub fn make_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

pub fn restore_terminal(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

pub struct UiState {
    pub scroll_state: ListState,
}

impl UiState {
    pub async fn input(&self, input: Event, state: &State) -> input::InputResult {
        if let Event::Key(key_event) = input {
            if key_event.kind == KeyEventKind::Release {
                return input::InputResult::Skip;
            }
            return input::match_keycode(
                &state.mode,
                state.files.get(state.selected).map(|f| f.path.clone()),
                key_event.code,
            );
        }
        input::InputResult::Skip
    }

    pub fn draw(&mut self, f: &mut Frame<'_, impl Backend>, state: &State) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(f.size());
        let left_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .split(layout[0]);

        self.draw_path(
            f,
            left_layout[0],
            state.path.to_str().wrap_err(UI_ERROR_WRAP).unwrap(),
        );
        self.draw_files(f, left_layout[1], state)
            .wrap_err(UI_ERROR_WRAP)
            .unwrap();
        self.draw_input(f, state);
        self.draw_info(f, left_layout[2], state);
        self.draw_content(f, layout[1], state);
    }

    fn draw_path(&mut self, f: &mut Frame<'_, impl Backend>, chunk: Rect, path: &str) {
        // Remove some windows stuff
        f.render_widget(Paragraph::new(path.replace("\\\\?\\", "")), chunk);
    }

    fn draw_files(
        &mut self,
        f: &mut Frame<'_, impl Backend>,
        chunk: Rect,
        state: &State,
    ) -> Result<()> {
        let mut items = state
            .files
            .iter()
            .enumerate()
            .map(|(pos, file)| {
                let text = file.name.clone().into_string().map_err(|s| {
                    eyre!(
                        "{UI_ERROR_WRAP}\nCould not convert filename {:?} to string",
                        s
                    )
                })?;

                let style = if pos == state.selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(self.get_file_color(file)?)
                } else {
                    Style::default().fg(self.get_file_color(file)?)
                };
                Ok(ListItem::new(text).style(style))
            })
            .collect::<Result<Vec<ListItem>, eyre::Error>>()
            .wrap_err(UI_ERROR_WRAP)
            .unwrap();

        if items.is_empty() {
            items.push(ListItem::new("No Files").style(Style::default().fg(Color::Gray)));
        }

        let list = List::new(items);
        self.scroll_state.select(Some(state.selected));
        f.render_stateful_widget(list, chunk, &mut self.scroll_state);
        Ok(())
    }

    fn get_file_color(&mut self, file: &File) -> Result<Color> {
        let kind = file.file_type;
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

    fn draw_input(&self, f: &mut Frame<'_, impl Backend>, state: &State) {
        if state.mode != Mode::Basic {
            let title = match &state.mode {
                Mode::Basic => "",
                Mode::CreateFile(_) => "Create File",
                Mode::RenameFile(_, _) => "Renaming file",
                Mode::DeleteFile(_, _) => "Deleting file",
            };

            let block = Block::default().title(title).borders(Borders::ALL);
            let p = Paragraph::new(state.mode.get_str().unwrap_or("")).block(block);
            let area = utils::centered_rect(60, 3, f.size());
            f.render_widget(Clear, area); //this clears out the background
            f.render_widget(p, area);
        }
    }

    fn draw_info(&mut self, f: &mut Frame<'_, impl Backend>, chunk: Rect, state: &State) {
        if let Some(i) = state.info.last() {
            let p = match &i.kind {
                InfoKind::Error(r) => Paragraph::new(
                    format!("{r}")
                        .split('\n')
                        .peekable()
                        .next()
                        .wrap_err(UI_ERROR_WRAP)
                        .unwrap()
                        .to_string(),
                )
                .style(Style::default().bg(Color::Red)),
                InfoKind::Message(s) => Paragraph::new(s.to_string()),
            };
            f.render_widget(p, chunk)
        } else {
            // do something if there is nothing here
        }
    }
    fn draw_content(&mut self, f: &mut Frame<'_, impl Backend>, chunk: Rect, state: &State) {
        let border = Block::default().borders(Borders::LEFT);
        let text = match state.files.get(state.selected).map(|f| f.file_type) {
            Some(f) if f.is_dir() => vec![Line::from("folder")],
            Some(f) if f.is_file() => {
                let ps = SyntaxSet::load_defaults_newlines();
                let ts = ThemeSet::load_defaults();
                let file = &state.files.get(state.selected).unwrap();
                let syntax = ps
                    .find_syntax_by_extension(
                        file.path.extension().unwrap_or_default().to_str().unwrap(),
                    )
                    .unwrap_or(ps.find_syntax_plain_text());

                let mut h = HighlightLines::new(syntax, &ts.themes["Solarized (dark)"]);
                let content = String::from_utf8(fs::read(&file.path).unwrap())
                    .unwrap_or("Binary".to_string());
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
            Some(f) if f.is_symlink() => vec![Line::from("symlink")],
            Some(_) => vec![Line::from("unknown")],
            None => vec![Line::from("Empty")],
        };
        let p = Paragraph::new(text).block(border);
        f.render_widget(p, chunk)
    }
}
