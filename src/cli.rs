use crossterm::style::Stylize;
use std::{
    collections::HashMap, iter, os::unix::process::ExitStatusExt, path::Path, process::ExitStatus,
};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use color_eyre::eyre::{ContextCompat, WrapErr, eyre};
use signal_hook::consts::{SIGINT, SIGTERM};
use tokio::{process::Command, signal};

use crate::trace_dbg;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Default)]
pub enum State {
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "paused")]
    Paused,
    #[serde(rename = "restarting")]
    Restarting,
    #[default]
    #[serde(rename = "exited")]
    Exited,
    #[serde(rename = "removing")]
    Removing,
    #[serde(rename = "dead")]
    Dead,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct ComposeData {
    #[serde(rename = "ConfigFiles")]
    pub config_files: String,
    #[serde(rename = "Status")]
    pub status: String,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(untagged)]
pub enum ProjectKind {
    Compose(ComposeData),
    #[serde(skip)]
    Standalone,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct ProjectContainersState {
    pub state: State,
    pub amount: usize,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Project {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(flatten)]
    pub kind: ProjectKind,
    #[serde(skip)]
    pub containers: Vec<Container>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct ContainerPlatform {
    pub architecture: String,
    pub os: String,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Container {
    #[serde(rename = "Command")]
    pub command: String,
    #[serde(rename = "CreatedAt")]
    pub created_at: String,
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Image")]
    pub image: String,
    #[serde(rename = "Labels")]
    pub labels_raw: String,
    #[serde(skip)]
    pub labels: HashMap<String, String>,
    #[serde(rename = "LocalVolumes")]
    pub local_volumes: String,
    #[serde(rename = "Mounts")]
    pub mounts: String,
    #[serde(rename = "Names")]
    pub names: String,
    #[serde(rename = "Networks")]
    pub networks: String,
    #[serde(rename = "Platform")]
    pub platform: Option<ContainerPlatform>,
    #[serde(rename = "Ports")]
    pub ports: String,
    #[serde(rename = "RunningFor")]
    pub running_for: String,
    #[serde(rename = "Size")]
    pub size: String,
    #[serde(rename = "State")]
    pub state: State,
    #[serde(rename = "Status")]
    pub status: String,
}

impl Container {
    pub fn title(&self) -> &str {
        if let Some(service) = self.labels.get("com.docker.compose.service") {
            service
        } else {
            &self.names
        }
    }
}

impl ProjectKind {
    pub fn as_compose(&self) -> Option<&ComposeData> {
        match self {
            ProjectKind::Compose(compose) => Some(compose),
            ProjectKind::Standalone => None,
        }
    }
}

impl Project {
    fn folder(&self) -> Option<String> {
        let config_files = self.kind.as_compose()?.config_files.clone();
        let file = config_files.split(",").next()?.to_string();
        Path::new(&file).parent()?.to_str().map(|v| v.to_string())
    }

    pub fn state(&self) -> State {
        if self.containers.iter().any(|c| c.state == State::Running) {
            return State::Running;
        }

        State::Exited
    }
}

pub async fn cli_action(
    cmd: impl Into<&str>,
    args: impl IntoIterator<Item = &str>,
) -> color_eyre::Result<String> {
    let output = Command::new(cmd.into()).args(args).output().await?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(eyre!("{}", err))
    } else {
        let out = String::from_utf8(output.stdout)
            .wrap_err("docker container list returned invalid utf8")?;
        Ok(out)
    }
}

pub async fn docker_get_projects() -> color_eyre::Result<Vec<Project>> {
    let output = cli_action("docker", ["compose", "ls", "-a", "--format", "json"]).await?;

    let mut projects = serde_json::from_str::<Vec<Project>>(&output)
        .wrap_err("Could not parse docker compose ls output")?;

    let conatiners_output =
        cli_action("docker", ["container", "ls", "-a", "--format", "json"]).await?;

    let parts = format!("[{}]", conatiners_output.lines().join(","));

    let containers = serde_json::from_str::<Vec<Container>>(&parts)
        .wrap_err("Could not parse docker container list output")?;

    let mut containers_by_project: HashMap<Option<String>, Vec<Container>> = HashMap::new();

    for mut container in containers {
        container.labels = container
            .labels_raw
            .split(",")
            .filter(|s| !s.is_empty())
            .filter_map(|key_value| {
                let (key, value) = key_value.split_once("=")?;
                Some((key.to_string(), value.to_string()))
            })
            .collect();

        let project_name = container.labels.get("com.docker.compose.project").cloned();

        containers_by_project
            .entry(project_name)
            .or_default()
            .push(container);
    }

    for project in &mut projects {
        project.containers = containers_by_project
            .remove(&Some(project.name.clone()))
            .unwrap_or_default()
    }

    if let Some(standalone) = containers_by_project.remove(&None) {
        projects.push(Project {
            name: "(standalone)".to_string(),
            kind: ProjectKind::Standalone,
            containers: standalone,
        })
    }

    Ok(projects)
}

pub async fn docker_project_action(
    project: Project,
    args: impl IntoIterator<Item = &str>,
) -> color_eyre::Result<String> {
    let folder = project
        .folder()
        .wrap_err_with(|| format!("folder for project '{}' not found", project.name))?;

    let output = Command::new("docker")
        .current_dir(folder)
        .args(args)
        .output()
        .await?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(eyre!("{}", err))
    } else {
        let out = String::from_utf8(output.stdout)
            .wrap_err("docker container list returned invalid utf8")?;
        Ok(out)
    }
}

pub async fn docker_action_tty(
    project: Project,
    args: impl IntoIterator<Item = &str>,
) -> color_eyre::Result<ExitStatus> {
    let folder = project
        .folder()
        .wrap_err_with(|| format!("folder for project '{}' not found", project.name))?;

    let args: Vec<_> = args.into_iter().collect();

    let cmd = iter::once("docker")
        .chain(args.iter().copied())
        .collect::<Vec<_>>()
        .join(" ");

    println!("\n{}", format!("\n--- Executing {:?} ---", cmd).cyan());

    let status = std::process::Command::new("docker")
        .current_dir(folder)
        .args(&args)
        .status()?;

    let was_interrupted = status.signal() == Some(SIGINT) || status.code() == Some(130);

    if !was_interrupted {
        println!(
            "\n{}",
            format!(
                "--- Command exited (code: {:?}) - Press Ctrl+C to return ---",
                status.code()
            )
            .cyan(),
        );

        signal::ctrl_c().await?;
    }

    Ok(status)
}
