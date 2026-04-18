use ratatui::{
    Frame,
    layout::Rect,
    style::Color,
    text::Line,
    widgets::{Block, BorderType, Borders, Paragraph},
};

use std::collections::BTreeMap;

use crate::{
    cli::{Container, Project, State},
    inspect::{HealthStatus, MountKind},
    model::Model,
};

struct Section {
    title: String,
    infos: Vec<String>,
}

impl Section {
    fn new(title: impl Into<String>, infos: Vec<String>) -> Self {
        Section {
            title: title.into(),
            infos,
        }
    }
}

fn section_lines(section: &Section, width: usize) -> Vec<Line<'static>> {
    let title = format!("{} ", section.title);
    let fill = width.saturating_sub(title.len() + 3);
    let header = format!("── {}{:─<fill$}", title, "");
    let mut lines = vec![Line::from(header)];
    lines.extend(section.infos.iter().cloned().map(Line::from));
    lines.push(Line::from(""));
    lines
}

pub fn render_project_info(model: &Model, project: &Project, frame: &mut Frame, area: Rect) {
    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(project.name.clone())
        .border_style(Color::White);

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let total = project.containers.len();
    let running = project
        .containers
        .iter()
        .filter(|c| c.state == State::Running)
        .count();

    let unhealthy = project
        .containers
        .iter()
        .filter(|c| {
            let health_bad = c
                .inspect(model)
                .and_then(|i| i.state.health.as_ref())
                .map(|h| h.status == HealthStatus::Unhealthy)
                .unwrap_or(false);
            let exit_bad = c
                .inspect(model)
                .map(|i| c.state == State::Exited && i.state.exit_code != 0)
                .unwrap_or(false);
            health_bad || exit_bad
        })
        .count();

    let file = project
        .kind
        .as_compose()
        .map(|c| c.config_files.as_str())
        .unwrap_or("(standalone)");

    let max_name = project
        .containers
        .iter()
        .map(|container| container.names.len())
        .max()
        .unwrap_or(0);

    let containers: Vec<_> = project
        .containers
        .iter()
        .map(|container| format!("{:<max_name$} {}", container.names, container.ports))
        .collect();

    let mut networks: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for container in &project.containers {
        let Some(inspect) = container.inspect(model) else {
            continue;
        };
        for net_name in inspect.network_settings.networks.keys() {
            networks
                .entry(net_name.clone())
                .or_default()
                .push(container.title().to_string());
        }
    }

    let max_net = networks.keys().map(|n| n.len()).max().unwrap_or(0);
    let network_lines: Vec<_> = networks
        .iter()
        .map(|(name, conts)| format!("{:<max_net$} → {}", name, conts.join(", ")))
        .collect();

    let mut volumes: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for container in &project.containers {
        let Some(inspect) = container.inspect(model) else {
            continue;
        };
        for mount in &inspect.mounts {
            if mount.kind == MountKind::Volume {
                volumes
                    .entry(mount.name.clone())
                    .or_default()
                    .push(container.title().to_string());
            }
        }
    }

    let max_vol = volumes.keys().map(|n| n.len()).max().unwrap_or(0);
    let volume_lines: Vec<_> = volumes
        .iter()
        .map(|(name, conts)| format!("{:<max_vol$} → {}", name, conts.join(", ")))
        .collect();

    let mut lines: Vec<Line<'static>> = vec![
        Line::from(format!(
            "Status: {running}/{total} running · {unhealthy} unhealthy"
        )),
        Line::from(format!("File:   {file}")),
        Line::from(""),
    ];

    for section in [
        Section::new("Containers", containers),
        Section::new("Networks", network_lines),
        Section::new("Volumes", volume_lines),
    ] {
        lines.extend(section_lines(&section, inner_area.width as usize));
    }

    let max_scroll = (lines.len() as u16).saturating_sub(inner_area.height);
    let scroll = model.info_scroll.min(max_scroll);

    frame.render_widget(
        Paragraph::new(lines)
            .style(Color::White)
            .scroll((scroll, 0)),
        inner_area,
    );
}

pub fn render_container_info(
    _model: &Model,
    _container: &Container,
    _frame: &mut Frame,
    _area: Rect,
) {
}

pub fn view(model: &mut Model, frame: &mut Frame, area: Rect) {
    let Some(project_index) = model.active_project_index else {
        return;
    };

    let project = &model.projects[project_index];

    if let Some(container_idx) = model.active_container_index {
        let container = &project.containers[container_idx];
        render_container_info(model, container, frame, area);
    } else {
        render_project_info(model, project, frame, area);
    }
}
