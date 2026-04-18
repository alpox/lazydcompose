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
    if section.infos.is_empty() {
        return Vec::new();
    }
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
        .title(format!("{} info", project.name.clone()))
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

pub fn render_container_info(model: &Model, container: &Container, frame: &mut Frame, area: Rect) {
    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(format!("{} info", container.title()))
        .border_style(Color::White);

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let inspect = container.inspect(model);

    let image = inspect
        .map(|i| i.config.image.as_str())
        .unwrap_or(container.image.as_str());

    let status = format!("{} · {}", container.status, container.running_for);

    let health = inspect
        .and_then(|i| i.state.health.as_ref())
        .map(|h| format!("{:?}", h.status))
        .unwrap_or_else(|| "—".to_string());

    let command = inspect
        .and_then(|i| {
            let entry = i.config.entrypoint.as_ref().map(|v| v.join(" "));
            let cmd = i.config.cmd.as_ref().map(|v| v.join(" "));
            match (entry, cmd) {
                (Some(e), Some(c)) => Some(format!("{e} {c}")),
                (Some(e), None) => Some(e),
                (None, Some(c)) => Some(c),
                _ => None,
            }
        })
        .unwrap_or_else(|| container.command.clone());

    let restarts = inspect.map(|i| i.restart_count).unwrap_or(0);

    let port_lines: Vec<String> = inspect
        .map(|i| {
            let mut ports: Vec<_> = i.network_settings.ports.iter().collect();
            ports.sort_by(|a, b| a.0.cmp(b.0));
            ports
                .into_iter()
                .flat_map(|(container_port, bindings)| {
                    let port = container_port.clone();
                    bindings
                        .as_deref()
                        .unwrap_or(&[])
                        .iter()
                        .map(move |b| {
                            let host_ip = if b.host_ip.is_empty() {
                                "0.0.0.0"
                            } else {
                                b.host_ip.as_str()
                            };
                            format!("{}:{} → {}", host_ip, b.host_port, port)
                        })
                        .collect::<Vec<_>>()
                })
                .collect()
        })
        .unwrap_or_default();

    let mount_lines: Vec<String> = inspect
        .map(|i| {
            let max_src = i.mounts.iter().map(|m| m.source.len()).max().unwrap_or(0);
            let mut mounts: Vec<_> = i.mounts.iter().collect();
            mounts.sort_by(|a, b| a.destination.cmp(&b.destination));
            mounts
                .iter()
                .map(|m| {
                    let kind = match m.kind {
                        MountKind::Bind => "bind",
                        MountKind::Volume => "vol ",
                        MountKind::Tmpfs => "tmpfs",
                        MountKind::NamedPipe => "pipe",
                        MountKind::Cluster => "clst",
                    };
                    let rw = if m.rw { "rw" } else { "ro" };
                    format!("{} {:<max_src$} → {} {}", kind, m.source, m.destination, rw)
                })
                .collect()
        })
        .unwrap_or_default();

    let network_lines: Vec<String> = inspect
        .map(|i| {
            let max_name = i
                .network_settings
                .networks
                .keys()
                .map(|n| n.len())
                .max()
                .unwrap_or(0);
            let mut nets: Vec<_> = i.network_settings.networks.iter().collect();
            nets.sort_by(|a, b| a.0.cmp(b.0));
            nets.iter()
                .map(|(name, net)| format!("{:<max_name$} {}", name, net.ip_address))
                .collect()
        })
        .unwrap_or_default();

    let label_lines: Vec<String> = inspect
        .map(|i| {
            let mut labels: Vec<_> = i.config.labels.iter().collect();
            labels.sort_by(|a, b| a.0.cmp(b.0));
            let max_key = labels.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
            labels
                .iter()
                .map(|(k, v)| format!("{:<max_key$} = {}", k, v))
                .collect()
        })
        .unwrap_or_default();

    let mut lines: Vec<Line<'static>> = vec![
        Line::from(format!("Image:    {image}")),
        Line::from(format!("Status:   {status}")),
        Line::from(format!("Health:   {health}")),
        Line::from(format!("Restarts: {restarts}")),
        Line::from(format!("Command:  {command}")),
        Line::from(""),
    ];

    for section in [
        Section::new("Ports", port_lines),
        Section::new("Mounts", mount_lines),
        Section::new("Networks", network_lines),
        Section::new("Labels", label_lines),
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

pub fn view(model: &mut Model, frame: &mut Frame, area: Rect) {
    let Some(project_index) = model.active_project_index else {
        return;
    };

    let project = &model.projects[project_index];

    let local_idx = model
        .project_container_index(project_index, model.active_container_index)
        .filter(|idx| *idx < project.containers.len());

    if let Some(container_idx) = local_idx {
        let container = &project.containers[container_idx];
        render_container_info(model, container, frame, area);
    } else {
        render_project_info(model, project, frame, area);
    }
}
