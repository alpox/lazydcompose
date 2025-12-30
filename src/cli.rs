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
