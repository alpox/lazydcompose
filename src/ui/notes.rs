use std::cmp::min;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    widgets::{Block, Clear, Paragraph, Widget, Wrap},
};
use textwrap::{Options, wrap};

use crate::model::Note;

#[derive(Clone)]
pub struct Notes {
    notes: Vec<Note>,
    width_percent: usize,
    max_width: usize,
}

impl Default for Notes {
    fn default() -> Self {
        Self {
            notes: Default::default(),
            width_percent: 30,
            max_width: 60,
        }
    }
}

fn calculate_wrapped_height(text: &str, area: Rect) -> usize {
    wrap(text, Options::new(area.width as usize)).len()
}

impl Notes {
    pub fn new(notes: Vec<Note>) -> Self {
        Notes {
            notes,
            ..Default::default()
        }
    }

    fn calculate_popup_width(&self, area: Rect) -> usize {
        min(
            area.width as usize * self.width_percent / 100,
            self.max_width,
        ) + 4
    }

    fn calculate_popup_height(&self, area: Rect) -> usize {
        let popup_width = self.calculate_popup_width(area);
        self.notes
            .iter()
            .map(|note| {
                calculate_wrapped_height(
                    note.text.as_str(),
                    Rect {
                        width: popup_width as u16,
                        ..area
                    },
                ) + 2
            })
            .sum::<usize>()
    }

    fn popup_rect(&self, area: Rect) -> Rect {
        let popup_height = self.calculate_popup_height(area);
        let popup_width = self.calculate_popup_width(area);
        let vertical = Layout::vertical([Constraint::Length(popup_height as u16)]).flex(Flex::End);
        let horizontal =
            Layout::horizontal([Constraint::Length(popup_width as u16)]).flex(Flex::End);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }
}

impl Widget for Notes {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let rect = self.popup_rect(area);

        let mut current_y = rect.y;

        for note in self.notes {
            let content_height = calculate_wrapped_height(note.text.as_str(), rect);
            let block_height = content_height as u16 + 2;

            let block_area = Rect {
                y: current_y,
                height: block_height,
                ..rect
            };

            current_y = current_y.saturating_add(block_height as u16);

            let paragraph = Paragraph::new(note.text.as_str())
                .style(note.style)
                .block(Block::bordered().border_style(note.style))
                .wrap(Wrap { trim: true });

            Clear.render(block_area, buf);
            paragraph.render(block_area, buf);
        }
    }
}
