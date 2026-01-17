use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::Color,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget},
};

use crate::model::{self};

#[derive(Clone)]
pub struct Prompt {
    prompt: model::Prompt,
}

impl Prompt {
    pub fn new(prompt: model::Prompt) -> Self {
        Prompt { prompt }
    }
}

impl Widget for Prompt {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Color::Green)
            .title(self.prompt.title);

        let max_width = area.width / 10 * 7;
        let rows = self.prompt.text.len() as u16 / max_width.saturating_sub(2);

        let paragraph = Paragraph::new(self.prompt.text).block(block);

        let popup_rect = area.centered(Constraint::Max(max_width), Constraint::Min(rows + 2));

        Clear.render(popup_rect, buf);
        paragraph.render(popup_rect, buf);
    }
}
