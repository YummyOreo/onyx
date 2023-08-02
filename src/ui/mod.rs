use std::{fs::DirEntry, io};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use eyre::{eyre, Result};
use ratatui::{
    prelude::{Backend, Constraint, CrosstermBackend, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, ListState},
    Frame, Terminal,
};

use crate::Mode;

pub mod input;

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
    pub selected: usize,
    pub scroll_state: ListState,
    pub max: usize,
    pub mode: Mode,
}

impl UiState {
    pub async fn input(&self, input: Event, files: &[DirEntry]) -> input::InputResult {
        if let Event::Key(key_event) = input {
            if key_event.kind == KeyEventKind::Release {
                return input::InputResult::Skip;
            }
            return input::match_keycode(
                &self.mode,
                files.get(self.selected).expect("should be there").path(),
                key_event.code,
            );
        }
        input::InputResult::Skip
    }

    pub fn draw(&mut self, f: &mut Frame<'_, impl Backend>, files: &[DirEntry]) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Min(3), Constraint::Max(3)].as_ref())
            .split(f.size());

        self.draw_files(f, layout[0], files);
        self.draw_input(f, layout[1]);
    }

    fn draw_files(&mut self, f: &mut Frame<'_, impl Backend>, chunk: Rect, files: &[DirEntry]) {
        let items = files
            .iter()
            .map(|file| {
                let text = file
                    .file_name()
                    .into_string()
                    .map_err(|s| eyre!("Could not convert filename {:?} to string", s))?;
                Ok(ListItem::new(text))
            })
            .collect::<Result<Vec<ListItem>, eyre::Error>>()
            .unwrap();

        let block = Block::default().title("Files").borders(Borders::ALL);
        let list = List::new(items).block(block).highlight_style(Style::default().bg(Color::Gray));
        self.scroll_state.select(Some(self.selected));
        f.render_stateful_widget(list, chunk, &mut self.scroll_state)
    }

    fn draw_input(&self, f: &mut Frame<'_, impl Backend>, chunk: Rect) {
        let title = match &self.mode {
            Mode::Basic => "",
            Mode::CreateFile(_) => "Create File",
            Mode::RenameFile(_, _) => "Renaming file",
            Mode::DeleteFile(_, _) => "Deleting file",
        };
        let block = Block::default().title(title).borders(Borders::ALL);
        let p = Paragraph::new(self.mode.get_str().unwrap_or("")).block(block);
        f.render_widget(p, chunk);
    }
}
