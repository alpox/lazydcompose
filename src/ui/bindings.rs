use std::cmp::min;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Widget},
};

use crate::{bindings::BINDINGS, effect::Effect, event::Message, model::Model};

pub struct Bindings<'a> {
    model: &'a Model,
}

impl<'a> Bindings<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }
}

impl<'a> Widget for Bindings<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let panel_block = Block::new()
            .title("Scoped bindings")
            .borders(Borders::ALL)
            .border_style(Color::Cyan);

        let global_block = Block::new()
            .title("Global bindings")
            .borders(Borders::ALL)
            .border_style(Color::Cyan);

        let global_bindings = BINDINGS.global();
        let panel_bindings = BINDINGS.bindings_for(self.model);

        let listed_bindings = [global_bindings.clone(), panel_bindings.clone()].concat();

        let max_binding_len = listed_bindings
            .iter()
            .map(|binding| binding.display.len())
            .max()
            .unwrap_or(0) as u16;

        let max_binding_width = listed_bindings
            .iter()
            .map(|binding| max_binding_len as usize + binding.description.len() + 2)
            .max()
            .unwrap_or(0) as u16;

        let rows: Vec<_> = panel_bindings
            .iter()
            .map(|binding| {
                Row::new(vec![
                    Cell::from(binding.display),
                    Cell::from(binding.description),
                ])
            })
            .collect();

        let panel_table = Table::new(
            rows,
            [Constraint::Length(max_binding_len + 1), Constraint::Fill(1)],
        )
        .block(panel_block)
        .row_highlight_style(Style::new().bg(Color::Rgb(40, 40, 60)))
        .highlight_symbol(">");

        let rows: Vec<_> = global_bindings
            .iter()
            .map(|binding| {
                Row::new(vec![
                    Cell::from(binding.display),
                    Cell::from(binding.description),
                ])
            })
            .collect();

        let global_table = Table::new(
            rows,
            [Constraint::Length(max_binding_len + 1), Constraint::Fill(1)],
        )
        .block(global_block)
        .row_highlight_style(Style::new().bg(Color::Rgb(40, 40, 60)))
        .highlight_symbol(">");

        let hints = Paragraph::new("q: Quit, Esc: Quit").centered();

        let popup_area = area.centered(
            Constraint::Length(max_binding_width + 2),
            Constraint::Length(min(
                listed_bindings.len() as u16 + 4, // 2 * 2 block borders
                (area.height as f32 * 0.8) as u16,
            )),
        );

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(panel_bindings.len() as u16 + 2),
                Constraint::Length(global_bindings.len() as u16 + 2),
                Constraint::Length(1),
            ])
            .split(popup_area);

        Clear.render(popup_area, buf);
        panel_table.render(chunks[0], buf);
        global_table.render(chunks[1], buf);
        hints.render(chunks[2], buf);
    }
}

pub fn handle_key(model: &mut Model, key: KeyEvent) -> Effect<Message> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            model.active_overlay_context = None;
            Effect::None
        }
        _ => Effect::None,
    }
}
