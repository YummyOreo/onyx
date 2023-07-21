use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use eyre::{eyre, Result};
use ratatui::{
    backend::CrosstermBackend,
    prelude::Backend,
    widgets::{Block, Borders},
    Frame, Terminal,
};
use std::{io, thread, time::Duration};

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

struct App {}

impl App {
    pub async fn handle_input(&self, e: Event) -> Result<bool> {
        Ok(matches!(e, event::Event::Key(k) if k.code == KeyCode::Char('q')))
    }

    pub fn draw(&mut self, f: &mut Frame<'_, impl Backend>) {
        let size = f.size();
        let block = Block::default().title("Block").borders(Borders::ALL);
        f.render_widget(block, size);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = make_terminal()?;
    let mut app = App {};

    loop {
        terminal.draw(|f| app.draw(f))?;

        let event_ready = tokio::task::spawn_blocking(|| event::poll(Duration::from_millis(250)));
        if event_ready.await?? && app.handle_input(event::read()?).await? {
            break;
        }
    }

    // restore terminal
    restore_terminal(terminal)
}
