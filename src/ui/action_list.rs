use std::{cmp::min, collections::HashSet};

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Widget},
};

use crate::bindings::{Key, KeyAction};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Action {
    pub keys: Vec<Key>,
    pub display: &'static str,
    pub description: &'static str,
    pub action: KeyAction,
}

pub struct ActionList<'a> {
    actions: &'a HashSet<Action>,
}

impl<'a> ActionList<'a> {
    pub fn new(actions: &'a HashSet<Action>) -> Self {
        Self { actions }
    }
}

impl<'a> Widget for ActionList<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let panel_block = Block::new()
            .title("Scoped bindings")
            .borders(Borders::ALL)
            .border_style(Color::Cyan);

        let max_action_len = self
            .actions
            .iter()
            .map(|action| action.display.len())
            .max()
            .unwrap_or(0) as u16;

        let max_action_width = self
            .actions
            .iter()
            .map(|binding| max_action_len as usize + binding.description.len() + 2)
            .max()
            .unwrap_or(0) as u16;

        let rows: Vec<_> = self
            .actions
            .iter()
            .map(|action| {
                Row::new(vec![
                    Cell::from(action.display),
                    Cell::from(action.description),
                ])
            })
            .collect();

        let panel_table = Table::new(
            rows,
            [Constraint::Length(max_action_len + 1), Constraint::Fill(1)],
        )
        .block(panel_block)
        .row_highlight_style(Style::new().bg(Color::Rgb(40, 40, 60)))
        .highlight_symbol(">");

        let hints = Paragraph::new("q: Quit, Esc: Quit").centered();

        let popup_area = area.centered(
            Constraint::Length(max_action_width + 2),
            Constraint::Length(min(
                self.actions.len() as u16 + 4, // 2 * 2 block borders
                (area.height as f32 * 0.8) as u16,
            )),
        );

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(self.actions.len() as u16 + 2),
                Constraint::Length(1),
            ])
            .split(popup_area);

        Clear.render(popup_area, buf);
        panel_table.render(chunks[0], buf);
        hints.render(chunks[1], buf);
    }
}
