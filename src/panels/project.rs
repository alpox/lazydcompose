use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, BorderType, Borders},
};

use crate::{
    model::Model,
    panels::{
        containers::{self},
        info,
    },
    ui::colors::Colorize,
};

pub fn view(
    model: &mut Model,
    project_index: usize,
    is_active: bool,
    frame: &mut Frame,
    area: Rect,
) {
    let project = &model.projects[project_index];
    let rows = project.containers.len();

    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(if is_active {
            BorderType::Double
        } else {
            BorderType::Rounded
        })
        .title(project.name.clone())
        .border_style(project.colorize());

    let [projects, info] = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([Constraint::Length(rows as u16 + 2), Constraint::Fill(1)])
        .areas(area);

    let inner = block.inner(area);
    frame.render_widget(block, projects);

    containers::view(model, project_index, frame, inner);
    info::view(model, frame, info)
}
