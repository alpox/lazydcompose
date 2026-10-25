use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Color,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget, Wrap},
};

#[derive(Clone)]
pub struct Prompt<'a> {
    title: &'a str,
    text: &'a str,
}

impl<'a> Prompt<'a> {
    pub fn new(title: &'a str, text: &'a str) -> Self {
        Self { title, text }
    }
}

impl<'a> Widget for Prompt<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Color::Green)
            .title(self.title);

        let max_width = area.width / 10 * 7;
        let rows = (self.text.len() as u16).div_ceil(max_width.saturating_sub(2));

        let paragraph = Paragraph::new(self.text)
            .block(block)
            .wrap(Wrap { trim: true });

        let popup_rect = area.centered(Constraint::Max(max_width), Constraint::Length(rows + 3));
        let [block_area, bindings] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(popup_rect);

        let hints = Paragraph::new("Esc: Cancel, Enter: Accept")
            .centered()
            .style(Color::White);

        Clear.render(popup_rect, buf);
        paragraph.render(block_area, buf);
        hints.render(bindings, buf);
    }
}
