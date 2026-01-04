use std::path::Path;

use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};

use color_eyre::{
    Section,
    eyre::{ContextCompat, Error, WrapErr, eyre},
};
use tokio::process::Command;

use crate::trace_dbg;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum State {
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "paused")]
    Paused,
    #[serde(rename = "restarting")]
    Restarting,
    #[serde(rename = "exited")]
    Exited,
    #[serde(rename = "removing")]
    Removing,
    #[serde(rename = "dead")]
    Dead,
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
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "ConfigFiles")]
    pub config_files: String,
    #[serde(skip)]
    pub state: Vec<ProjectContainersState>,
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
    pub labels: String,
    #[serde(rename = "LocalVolumes")]
    pub local_volumes: String,
    #[serde(rename = "Mounts")]
    pub mounts: String,
    #[serde(rename = "Names")]
    pub names: String,
    #[serde(rename = "Networks")]
    pub networks: String,
    #[serde(rename = "Platform")]
    pub platform: ContainerPlatform,
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

impl Project {
    fn folder(&self) -> Option<String> {
        let file = self.config_files.split(",").next()?.to_string();
        Path::new(&file).parent()?.to_str().map(|v| v.to_string())
    }
}

pub async fn docker_compose_ls() -> color_eyre::Result<Vec<Project>> {
    let output = Command::new("docker")
        .args(["compose", "ls", "-a", "--format", "json"])
        .output()
        .await?;

    let utf8_output =
        String::from_utf8(output.stdout).wrap_err("Docker compose ls returned invalid utf8")?;

    let mut projects = serde_json::from_str::<Vec<Project>>(&utf8_output)
        .wrap_err("Could not parse docker compose ls output")?;

    let reg = Regex::new(r"(\w+)\((\d+)\)").unwrap();

    for project in &mut projects {
        project.state = reg
            .captures_iter(&project.status)
            .map(|capture| capture.extract())
            .flat_map(|(_, [state, amount])| {
                Some(ProjectContainersState {
                    state: serde_json::from_str::<State>(&format!("\"{}\"", state)).ok()?,
                    amount: amount.parse::<usize>().ok()?,
                })
            })
            .collect();
    }

    Ok(projects)
}

pub async fn docker_container_list(
    args: impl IntoIterator<Item = &str>,
) -> color_eyre::Result<Vec<Container>> {
    let mut cmd_args = vec!["container", "list", "--format", "json"];
    cmd_args.extend(args);

    let output = Command::new("docker").args(cmd_args).output().await?;

    let utf8_output =
        String::from_utf8(output.stdout).wrap_err("docker container list returned invalid utf8")?;

    let parts = format!("[{}]", utf8_output.lines().join(","));

    serde_json::from_str::<Vec<Container>>(&parts)
        .wrap_err("Could not parse docker container list output")
}

pub async fn docker_compose_action(
    project: Project,
    args: impl IntoIterator<Item = &str>,
) -> color_eyre::Result<String> {
    let mut cmd_args = vec!["compose"];
    cmd_args.extend(args);

    let folder = project
        .folder()
        .wrap_err_with(|| format!("folder for project '{}' not found", project.name))?;

    let output = Command::new("docker")
        .current_dir(folder)
        .args(cmd_args)
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
