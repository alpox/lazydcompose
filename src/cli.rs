use crossterm::style::Stylize;
use std::{os::unix::process::ExitStatusExt, process::ExitStatus};

use color_eyre::eyre::{WrapErr, eyre};
use signal_hook::consts::SIGINT;
use tokio::{process::Command, signal};

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

