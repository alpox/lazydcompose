use color_eyre::eyre::Context;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

use crate::{data::inspect::ContainerInspect, model::Model};

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

impl ProjectKind {
    pub fn as_compose(&self) -> Option<&ComposeData> {
        match self {
            ProjectKind::Compose(compose) => Some(compose),
            ProjectKind::Standalone => None,
        }
    }
}

impl Project {
    pub fn parse(ls_json: String, containers_json: String) -> color_eyre::Result<Vec<Project>> {
        let mut projects = serde_json::from_str::<Vec<Project>>(&ls_json)
            .wrap_err("Could not parse docker compose ls output")?;

        let containers = serde_json::Deserializer::from_str(&containers_json)
            .into_iter::<Container>()
            .collect::<Result<Vec<_>, _>>()
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

    pub fn folder(&self) -> Option<String> {
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

impl Container {
    pub fn title(&self) -> &str {
        if let Some(service) = self.labels.get("com.docker.compose.service") {
            service
        } else {
            &self.names
        }
    }

    pub fn inspect<'a>(&self, model: &'a Model) -> Option<&'a ContainerInspect> {
        model.inspects.get(&self.id)
    }
}
