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
    fs::{self, ReadDir},
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

struct App {
    files: ReadDir,
    path: PathBuf,
    selected: usize,
    max: usize,
}

impl App {
    pub async fn handle_input(&mut self, e: Event) -> Result<bool> {
        Ok(if let event::Event::Key(k) = e {
            if k.kind == KeyEventKind::Release {
                return Ok(false);
            }
            match k.code {
                KeyCode::Char('q') => true,
                KeyCode::Up | KeyCode::Char('k') => {
                    self.selected = self.selected.checked_sub(1).unwrap_or_default();
                    false
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.selected += 1;
                    self.selected = self.selected.clamp(0, self.max);
                    false
                }
                _ => false,
            }
        } else {
            false
        })
    }

    pub fn draw(&mut self, f: &mut Frame<'_, impl Backend>) -> Result<()> {
        let mut items = vec![];
        let mut pos = 0;
        for file in &mut self.files {
            let text = file?
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
        let pblock = Block::default().title("Info").borders(Borders::ALL);
        let p = Paragraph::new("").block(pblock);
        f.render_widget(p, chunks[1]);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = make_terminal()?;
    let mut app = App {
        files: fs::read_dir("./")?,
        path: PathBuf::from("./"),
        selected: 0,
        max: 0,
    };

    loop {
        app.files = fs::read_dir(&app.path)?;
        terminal.draw(|f| app.draw(f).unwrap())?;

        let event_ready = tokio::task::spawn_blocking(|| event::poll(Duration::from_millis(250)));
        if event_ready.await?? && app.handle_input(event::read()?).await? {
            break;
        }
    }

    // restore terminal
    restore_terminal(terminal)
}
