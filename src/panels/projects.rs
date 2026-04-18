use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::{cli::Project, model::Model, panels::project};

pub fn view(model: &mut Model, frame: &mut Frame, area: Rect) {
    let min_size = 4_u16;
    let project_height = |project: &Project| (project.containers.len() as u16 + 2).min(min_size);
    let project_index_height =
        |idx: usize| model.projects.get(idx).map(project_height).unwrap_or(2);
    let mut offsets = model
        .projects
        .iter()
        .scan(0, |sum, project| {
            *sum += project_height(project);
            Some(*sum)
        })
        .collect::<Vec<_>>();

    // First project offset
    offsets.insert(0, 0);

    // Min block size of 4 per project (2 rows of containers)
    let min_total_height = *offsets.last().unwrap_or(&0);
    let needs_scroll = area.height < min_total_height;

    let active_idx = model.active_project_index.unwrap_or(0);
    let active_y = offsets.get(active_idx).unwrap_or(&0);
    let scroll_offset = (active_y + project_index_height(active_idx)).saturating_sub(area.height);

    let first_visible_index = offsets
        .iter()
        .position(|&offset| scroll_offset <= offset)
        .unwrap_or(0);
    let first_visible_offset = offsets.get(first_visible_index).copied().unwrap_or(0);

    let visible_projects: Vec<_> = model.projects.iter().skip(first_visible_index).collect();

    let layout = if needs_scroll {
        visible_projects
            .iter()
            .enumerate()
            .map(|(i, _)| Rect {
                height: project_index_height(first_visible_index + i),
                y: area.y + offsets[i + first_visible_index] - first_visible_offset,
                ..area
            })
            .filter(|rect| (rect.y + rect.height) <= (area.y + area.height))
            .collect::<Vec<_>>()
    } else {
        let constraints = visible_projects
            .iter()
            .map(|project| Constraint::Length(project.containers.len() as u16 + 2));

        Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area)
            .to_vec()
    };

    for (i, area) in layout.iter().enumerate() {
        let project_index = i + first_visible_index;
        let is_active = model
            .active_project_index
            .map(|idx| (idx - first_visible_index) == i)
            .unwrap_or(false);

        project::view(model, project_index, is_active, frame, *area);
    }
}
