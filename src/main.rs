use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use eyre::{eyre, Result};
use ratatui::{
    backend::CrosstermBackend,
    prelude::{Backend, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::{
    fs::{self, DirEntry, ReadDir},
    io,
    path::PathBuf,
    time::Duration,
};

mod settings;

fn make_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

pub enum Mode {
    Basic,
    CreateFile(String),
    RenameFile(PathBuf),
    DeleteFile(PathBuf),
}

impl Default for Mode {
    fn default() -> Self {
        Self::Basic
    }
}

#[derive(PartialEq, Eq)]
enum InputResult {
    Quit,
    None,
}

struct App {
    files: Vec<DirEntry>,
    path: PathBuf,
    selected: usize,
    max: usize,
    mode: Mode,
}

impl App {
    pub async fn handle_input(&mut self, e: Event) -> InputResult {
        if let event::Event::Key(k) = e {
            if k.kind == KeyEventKind::Release {
                return InputResult::None;
            }
            match k.code {
                KeyCode::Char('q') => {
                    return InputResult::Quit;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.selected = self.selected.checked_sub(1).unwrap_or_default();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.selected += 1;
                    self.selected = self.selected.clamp(0, self.max);
                }
                KeyCode::Char('c') => self.mode = Mode::CreateFile(String::new()),
                KeyCode::Char('r') => {
                    self.mode = Mode::RenameFile(self.files.get(self.selected).unwrap().path())
                }
                KeyCode::Char('d') => {
                    self.mode = Mode::DeleteFile(self.files.get(self.selected).unwrap().path())
                }
                KeyCode::Esc => self.mode = Mode::Basic,
                _ => (),
            }
        }
        InputResult::None
    }

    pub fn draw(&mut self, f: &mut Frame<'_, impl Backend>) -> Result<()> {
        let mut items = vec![];
        let mut pos = 0;
        for file in &mut self.files {
            let text = file
                .file_name()
                .into_string()
                .map_err(|s| eyre!("Could not convert filename {:?} to string", s))?;
            let color = if pos == self.selected {
                Color::Gray
            } else {
                Color::Reset
            };
            let item = ListItem::new(text).style(Style::new().bg(color));
            items.push(item);
            pos += 1;
        }
        self.max = pos - 1;

        let listblock = Block::default().title("Fs").borders(Borders::ALL);
        let list = List::new(items).block(listblock);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
            .split(f.size());
        f.render_widget(list, chunks[0]);

        // MODE STUFF
        let title = match &self.mode {
            Mode::Basic => "",
            Mode::CreateFile(_) => "Create File",
            Mode::RenameFile(_) => "Renaming file",
            Mode::DeleteFile(_) => "Deleting file",
        };
        let pblock = Block::default().title(title).borders(Borders::ALL);
        let p = Paragraph::new("").block(pblock);
        f.render_widget(p, chunks[1]);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = make_terminal()?;
    let mut app = App {
        files: fs::read_dir("./")?.map(|f| f.unwrap()).collect(),
        path: PathBuf::from("./"),
        selected: 0,
        max: 0,
        mode: Mode::default(),
    };

    loop {
        app.files = fs::read_dir(&app.path)?.map(|f| f.unwrap()).collect();
        terminal.draw(|f| app.draw(f).unwrap())?;

        let event_ready = tokio::task::spawn_blocking(|| event::poll(Duration::from_millis(250)));
        if event_ready.await?? && app.handle_input(event::read()?).await == InputResult::Quit {
            break;
        }
    }

    // restore terminal
    restore_terminal(terminal)
}
