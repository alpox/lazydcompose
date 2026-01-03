use itertools::Itertools;
use serde::{Deserialize, Serialize};

use color_eyre::eyre::WrapErr;
use tokio::process::Command;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Project {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "ConfigFiles")]
    pub config_files: String,
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
    pub state: String,
    #[serde(rename = "Status")]
    pub status: String,
}

pub async fn docker_compose_ls() -> color_eyre::Result<Vec<Project>> {
    let output = Command::new("docker")
        .args(["compose", "ls", "-a", "--format", "json"])
        .output()
        .await?;

    let utf8_output =
        String::from_utf8(output.stdout).wrap_err("Docker compose ls returned invalid utf8")?;

    serde_json::from_str::<Vec<Project>>(&utf8_output)
        .wrap_err("Could not parse docker compose ls output")
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
