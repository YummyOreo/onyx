use crate::state::{InfoKind, Mode, State};
use eyre::ContextCompat;
use ratatui::{
    prelude::{Backend, Rect},
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};

use super::UI_ERROR_WRAP;

pub fn render_info_line(f: &mut Frame<'_, impl Backend>, chunk: Rect, state: &State) {
    if matches!(state.mode, Mode::Search(_)) {
        let p = Paragraph::new(state.mode.get().unwrap_or_default());
        f.render_widget(p, chunk);
        return;
    }
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
