use crossterm::style::Stylize;
use std::{collections::HashMap, os::unix::process::ExitStatusExt, process::ExitStatus};

use itertools::Itertools;

use color_eyre::eyre::{WrapErr, eyre};
use signal_hook::consts::SIGINT;
use tokio::{process::Command, signal};

use crate::data::compose::{Container, Project, ProjectKind};

#[derive(Clone, Debug, PartialEq)]
pub struct CliCommand {
    pub program: &'static str,
    pub args: Vec<String>,
    pub cwd: Option<String>,
}

impl CliCommand {
    pub fn docker(args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            program: "docker",
            args: args.into_iter().map(Into::into).collect(),
            cwd: None,
        }
    }

    pub fn cwd(mut self, cwd: Option<String>) -> Self {
        self.cwd = cwd;
        self
    }

    pub async fn run(&self) -> color_eyre::Result<String> {
        let mut cmd = Command::new(self.program);

        cmd.args(self.args.iter().map(String::as_str));

        if let Some(cwd) = &self.cwd {
            cmd.current_dir(cwd);
        }

        let output = cmd.output().await?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            Err(eyre!("{}", err))
        } else {
            let out = String::from_utf8(output.stdout)
                .wrap_err("docker container list returned invalid utf8")?;
            Ok(out)
        }
    }

    pub async fn run_tty(&self) -> color_eyre::Result<ExitStatus> {
        let mut cmd = std::process::Command::new(self.program);

        cmd.args(self.args.iter().map(String::as_str));

        if let Some(cwd) = &self.cwd {
            cmd.current_dir(cwd);
        }

        println!("\n{}", format!("\n--- Executing {:?} ---", cmd).cyan());

        let status = cmd.status()?;

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

    let containers_output =
        cli_action("docker", ["container", "ls", "-a", "--format", "json"]).await?;

    let parts = format!("[{}]", containers_output.lines().join(","));

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
