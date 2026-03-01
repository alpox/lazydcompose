use std::cmp::min;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{
        Block, Borders, Cell, Clear, Paragraph, Row, StatefulWidget, Table, TableState, Widget,
    },
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

        let panel_bindings_len = panel_bindings.iter().len();

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

        let mut panel_table_state = TableState::new();
        if let Some(idx) = self.model.selected_action_index
            && idx < panel_bindings_len
        {
            panel_table_state.select(self.model.selected_action_index);
        }

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

        let mut global_table_state = TableState::new();
        if let Some(idx) = self.model.selected_action_index
            && idx >= panel_bindings_len
        {
            global_table_state.select(Some(idx.saturating_sub(panel_bindings_len)));
        }

        let hints = Paragraph::new("q: Quit, Esc: Quit").centered();

        let popup_area = area.centered(
            Constraint::Length(max_binding_width + 2),
            Constraint::Length(min(
                listed_bindings.len() as u16 + 5, // 2 * 2 block borders + 1 hint
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
        StatefulWidget::render(panel_table, chunks[0], buf, &mut panel_table_state);
        StatefulWidget::render(global_table, chunks[1], buf, &mut global_table_state);
        hints.render(chunks[2], buf);
    }
}

pub fn handle_key(model: &mut Model, key: KeyEvent) -> Effect<Message> {
    let bindings = [BINDINGS.bindings_for(model), BINDINGS.global()].concat();
    let last_index = bindings.iter().len() - 1;

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            model.selected_action_index = match model.selected_action_index {
                Some(idx) if idx == last_index => Some(0),
                Some(idx) => Some(idx + 1),
                None => Some(0),
            };

            Effect::None
        }
        KeyCode::Char('k') | KeyCode::Up => {
            model.selected_action_index = match model.selected_action_index {
                Some(0) => Some(last_index),
                Some(idx) => Some(idx - 1),
                None => Some(last_index),
            };

            Effect::None
        }
        KeyCode::Enter => {
            if let Some(idx) = model.selected_action_index {
                model.selected_action_index = None;
                model.active_overlay_context = None;
                let binding = bindings.get(idx);
                if let Some(key) = binding.and_then(|binding| binding.keys.first()) {
                    Effect::dispatch(Message::KeyPress(key.into()))
                } else {
                    Effect::None
                }
            } else {
                Effect::None
            }
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            model.active_overlay_context = None;
            model.selected_action_index = None;
            Effect::None
        }
        _ => Effect::None,
    }
}
