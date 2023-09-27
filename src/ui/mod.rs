use std::io;

use crate::{state::InfoKind, State};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use eyre::{eyre, Context, ContextCompat, Result};
use ratatui::{
    prelude::{Backend, Constraint, CrosstermBackend, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

mod info_line;
pub mod input;
mod side_panel;
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
                key_event.code,
            );
        }
        input::InputResult::Skip
    }

    pub fn draw(&mut self, f: &mut Frame<'_, impl Backend>, state: &State) {
        let info_line_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(f.size());
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(info_line_layout[0]);
        let left_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
            .split(layout[0]);

        self.draw_path(
            f,
            left_layout[0],
            state.path.to_str().wrap_err(UI_ERROR_WRAP).unwrap(),
        );
        self.draw_files(f, left_layout[1], state)
            .wrap_err(UI_ERROR_WRAP)
            .unwrap();
        info_line::render_info_line(f, info_line_layout[1], state);
        side_panel::draw_side_panel(f, layout[1], state);
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
                        .bg(utils::get_file_color(&file.file_type))
                } else {
                    Style::default().fg(utils::get_file_color(&file.file_type))
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
}
