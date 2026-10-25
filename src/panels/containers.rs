use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Cell, ListItem, Row, Table, TableState},
};

use crate::{
    data::compose::Container,
    model::{ContextId, Model, ResourceId},
    ui::colors::Colorize,
};

impl From<&Container> for ListItem<'_> {
    fn from(value: &Container) -> Self {
        ListItem::new(Line::styled(
            format!("Container: {}", value.names),
            Color::Cyan,
        ))
    }
}

pub fn view(model: &mut Model, project_index: usize, frame: &mut Frame, area: Rect) {
    let project = &model.projects[project_index];

    let max_name_len = project
        .containers
        .iter()
        .map(|container| container.title().len())
        .max()
        .unwrap_or(0) as u16;

    let rows: Vec<_> = project
        .containers
        .iter()
        .map(|container| {
            let prefix = if model.has_pending_action(&ResourceId::Container(container.id.clone())) {
                "↻ "
            } else {
                ""
            };
            Row::new(vec![
                Cell::from(prefix.to_string() + container.title()),
                Cell::from(container.status.clone()),
            ])
            .style(container.colorize())
        })
        .collect();

    let table = Table::new(
        rows,
        [Constraint::Length(max_name_len + 1), Constraint::Fill(1)],
    )
    .row_highlight_style(Style::new().bg(Color::Rgb(50, 60, 90)).bold())
    .highlight_symbol("▶ ");

    let mut table_state = TableState::new();

    let is_active = model.active_context == ContextId::Containers
        && model
            .active_project_index
            .map(|active| active == project_index)
            .unwrap_or(false);

    if is_active {
        table_state
            .select(model.project_container_index(project_index, model.active_container_index));
    }

    frame.render_stateful_widget(table, area, &mut table_state);
}
