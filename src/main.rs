use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
    },
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
    fs::{self, DirEntry},
    io,
    path::PathBuf,
    time::Duration,
};

mod settings;
mod ui;

#[derive(PartialEq, Eq)]
pub enum Mode {
    Basic,
    CreateFile(String),
    RenameFile(PathBuf, String),
    DeleteFile(PathBuf, String),
}

impl Default for Mode {
    fn default() -> Self {
        Self::Basic
    }
}

impl Mode {
    pub fn add_char(&mut self, c: char) {
        match self {
            Self::CreateFile(s) | Self::RenameFile(_, s) | Mode::DeleteFile(_, s) => s.push(c),
            _ => {}
        }
    }
    pub fn remove_char(&mut self) {
        match self {
            Self::CreateFile(s) | Self::RenameFile(_, s) | Mode::DeleteFile(_, s) => {
                s.pop();
            }
            _ => {}
        }
    }
    pub fn get_str(&self) -> Option<&str> {
        match self {
            Self::CreateFile(s) | Self::RenameFile(_, s) | Mode::DeleteFile(_, s) => Some(s),
            _ => None,
        }
    }
}

#[derive(PartialEq, Eq)]
enum InputResult {
    Quit,
    Enter,
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
            if self.mode != Mode::Basic {
                return self.handle_input_mode(k).await;
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
                    self.mode = Mode::RenameFile(
                        self.files.get(self.selected).unwrap().path(),
                        String::new(),
                    )
                }
                KeyCode::Char('d') => {
                    self.mode = Mode::DeleteFile(
                        self.files.get(self.selected).unwrap().path(),
                        String::new(),
                    )
                }
                KeyCode::Esc => self.mode = Mode::Basic,
                _ => (),
            }
        }
        InputResult::None
    }

    pub async fn handle_input_mode(&mut self, k: KeyEvent) -> InputResult {
        match k.code {
            KeyCode::Char(c) => match self.mode {
                Mode::CreateFile(_) | Mode::RenameFile(_, _) | Mode::DeleteFile(_, _) => {
                    self.mode.add_char(c)
                }
                _ => {}
            },
            KeyCode::Enter => {
                return InputResult::Enter;
            }
            KeyCode::Esc => self.mode = Mode::Basic,
            KeyCode::Up => {
                self.selected = self.selected.checked_sub(1).unwrap_or_default();
            }
            KeyCode::Down => {
                self.selected += 1;
                self.selected = self.selected.clamp(0, self.max);
            }
            _ => {}
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
            Mode::RenameFile(_, _) => "Renaming file",
            Mode::DeleteFile(_, _) => "Deleting file",
        };
        let pblock = Block::default().title(title).borders(Borders::ALL);
        let p = Paragraph::new(self.mode.get_str().unwrap_or("")).block(pblock);
        f.render_widget(p, chunks[1]);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = ui::make_terminal()?;
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

        if event_ready.await?? {
            match app.handle_input(event::read()?).await {
                InputResult::Quit => {
                    break;
                }
                InputResult::Enter => match &app.mode {
                    Mode::CreateFile(file) => {
                        let path = app.path.join(file);
                        fs::create_dir_all(path)?;
                    }
                    Mode::RenameFile(f, n) => {
                        // dbg!(app.path.join(f));
                        fs::rename(app.path.join(f), n)?;
                    }
                    Mode::DeleteFile(f, i) => {
                        if i.to_lowercase() == 'y'.to_string() {
                            fs::remove_file(app.path.join(f))?;
                        }
                    }
                    _ => {}
                },
                InputResult::None => {}
            }
        }
    }

    // restore terminal
    ui::restore_terminal(terminal)
}
