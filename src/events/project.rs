use color_eyre::eyre::Context;
use futures::try_join;
use ratatui::style::{Color, Style};

use crate::{
    cli::CliCommand, data::{compose::Project, inspect::ContainerInspect}, effect::Effect, model::{Note, Prompt, ViewId}, system::{Ctx, Handler, Message}, util::ResultExt,
};

#[derive(Clone, Debug, PartialEq)]
pub struct SetProjects {
    projects: Vec<Project>,
}

fn note_styled(ctx: &mut Ctx, text: impl Into<String>, style: Style) -> Effect<Message> {
    ctx.db.notes.push(Note::new(text.into()).style(style));
    Effect::None
}

fn note_err(ctx: &mut Ctx, text: impl Into<String>) -> Effect<Message> {
    note_styled(ctx, text, Style::new().fg(Color::Red))
}

impl Handler for SetProjects {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.projects = self.projects;
        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SetProjectInfo {
    project_info: Result<(String, Vec<ContainerInspect>), String>,
}

impl Handler for SetProjectInfo {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        match self.project_info {
            Err(err) => note_err(ctx, err),
            Ok((_, inspects)) => {
                for inspect in inspects {
                    let key = inspect.id[..12].to_string();
                    ctx.db.inspects.insert(key, inspect);
                }

                Effect::None
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RefreshProjects;

impl Handler for RefreshProjects {
    fn handle(self, _ctx: &mut Ctx) -> Effect<Message> {
        Effect::perform(async move {
            let projects = CliCommand::docker(["compose", "ls", "-a", "--format", "json"]);
            let containers = CliCommand::docker(["container", "ls", "-a", "--format", "json"]);

            let result = try_join!(projects.run(), containers.run())
                .and_then(|(ls, containers)| Project::parse(ls, containers))
                .stringify_err();

            match result {
                Ok(projects) => Some(SetProjects { projects }.into()),
                Err(_) => None,
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RefreshProjectInfo {
    project_name: String,
}

impl Handler for RefreshProjectInfo {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        let ids: Vec<String> = ctx
            .db
            .projects
            .iter()
            .find(|p| p.name == self.project_name)
            .map(|p| p.containers.iter().map(|c| c.id.clone()).collect())
            .unwrap_or_default();

        if ids.is_empty() {
            return Effect::None;
        }

        Effect::perform(async move {
            let mut args = vec!["inspect".to_string()];
            args.extend(ids);
            let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();

            let result = CliCommand::docker(arg_refs)
                .run()
                .await
                .and_then(|out| {
                    serde_json::from_str::<Vec<ContainerInspect>>(&out)
                        .wrap_err("parse docker inspect")
                })
                .map(|inspects| (self.project_name, inspects))
                .stringify_err();

            Some(
                SetProjectInfo {
                    project_info: result,
                }
                .into(),
            )
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShowInfo;

impl Handler for ShowInfo {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.active_view = ViewId::Info;
        ctx.db.info_scroll = 0;

        match ctx.db.selected_project() {
            Some(p) => Effect::Dispatch(
                RefreshProjectInfo {
                    project_name: p.name,
                }
                .into(),
            ),
            None => Effect::None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct QuitInfo;

impl Handler for QuitInfo {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.active_view = ViewId::Main;
        ctx.db.info_scroll = 0;
        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollDown;

impl Handler for ScrollDown {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.info_scroll = ctx.db.info_scroll.saturating_add(1);
        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollUp;

impl Handler for ScrollUp {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.info_scroll = ctx.db.info_scroll.saturating_sub(1);
        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MoveUp;

impl Handler for MoveUp {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.select_previous_project();
        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MoveDown;

impl Handler for MoveDown {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.select_next_project();
        Effect::None
    }
}

// #[derive(Clone, Debug, PartialEq)]
// pub struct DockerComposeStop;
//
// impl Handler for DockerComposeStop {
//     fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
//         if let Some(project) = &ctx.db.selected_project() {
//             ctx.db.prompt(Prompt::new(
//                 "Confirm",
//                 format!(
//                     "Are you sure that you want to stop the project '{}'?",
//                     project.name
//                 ),
//                 docker_compose_action(model, PendingOperation::Stopping, |_| {
//                     args(["compose", "stop"])
//                 }),
//             ))
//         }
//
//         Effect::None
//     }
// }
