use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use eyre::Result;
use ratatui::{
    backend::CrosstermBackend,
    prelude::Backend,
    style::{Color, Style},
    text::Text,
    widgets::{List, ListItem},
    Frame, Terminal,
};
use std::{
    fs::{self, ReadDir},
    io,
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

    pub fn draw(&mut self, f: &mut Frame<'_, impl Backend>, files: ReadDir) {
        let size = f.size();
        let mut items = vec![];
        let mut pos = 0;
        for file in files {
            let text = file.unwrap().file_name().into_string().unwrap();
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
        let list = List::new(items);
        f.render_widget(list, size);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = make_terminal()?;
    let mut app = App {
        selected: 0,
        max: 0,
    };

    loop {
        let files = fs::read_dir("./")?;
        terminal.draw(|f| app.draw(f, files))?;

        let event_ready = tokio::task::spawn_blocking(|| event::poll(Duration::from_millis(250)));
        if event_ready.await?? && app.handle_input(event::read()?).await? {
            break;
        }
    }

    // restore terminal
    restore_terminal(terminal)
}
