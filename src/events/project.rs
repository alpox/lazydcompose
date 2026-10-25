use color_eyre::eyre::Context;
use futures::try_join;
use ratatui::style::{Color, Style};

use crate::{
    cli::CliCommand, data::{compose::Project, inspect::ContainerInspect}, effect::Effect, events::general::Batch, model::{ContextId, Note, PendingOperation, Prompt, ResourceId, ViewId}, system::{Ctx, Handler, Message}, util::{ResultExt, args},
};

fn note_styled(ctx: &mut Ctx, text: impl Into<String>, style: Style) -> Effect<Message> {
    ctx.db.notes.push(Note::new(text.into()).style(style));
    Effect::None
}

fn note_err(ctx: &mut Ctx, text: impl Into<String>) -> Effect<Message> {
    note_styled(ctx, text, Style::new().fg(Color::Red))
}

pub fn docker_compose_action<F, R>(project: Project, op: PendingOperation, f: F) -> Effect<Message>
where
    F: FnOnce(Project) -> R + Send + 'static,
    R: IntoIterator<Item = String> + Send,
{
    let mut effects: Vec<_> = project
        .containers
        .iter()
        .map(|container| {
            Effect::Dispatch(
                InitPendingAction {
                    resource_id: ResourceId::Container(container.id.clone()),
                    pending_op: op.clone(),
                }
                .into(),
            )
        })
        .collect();

    let async_effect = Effect::perform(async move {
        let args: Vec<_> = f(project.clone()).into_iter().collect();
        let result = CliCommand::docker(args).cwd(project.folder()).run().await;

        match result {
            Ok(_) => Some(UpdatePendingActions.into()),
            Err(err) => Some(
                Batch::new([
                    UpdatePendingActions.into(),
                    NoteErr {
                        text: err.to_string(),
                    }
                    .into(),
                ])
                .into(),
            ),
        }
    });

    effects.push(async_effect);

    Effect::Batch(effects)
}

#[derive(Clone, Debug, PartialEq)]
pub struct NoteErr {
    pub text: String,
}

impl Handler for NoteErr {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        note_err(ctx, self.text)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InitPendingAction {
    pub resource_id: ResourceId,
    pub pending_op: PendingOperation,
}

impl Handler for InitPendingAction {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db
            .init_pending_action(self.resource_id, self.pending_op);
        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UpdatePendingActions;

impl Handler for UpdatePendingActions {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.update_pending_actions();
        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SetProjects {
    projects: Vec<Project>,
}

impl Handler for SetProjects {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.projects = self.projects;
        if !ctx.db.projects.is_empty() && ctx.db.active_project_index.is_none() {
            ctx.db.active_project_index = Some(0);
        }
        ctx.db.update_pending_actions();
        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SetProjectInfo {
    project_info: (String, Vec<ContainerInspect>),
}

impl Handler for SetProjectInfo {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        let (_, inspects) = self.project_info;

        for inspect in inspects {
            let key = inspect.id[..12].to_string();
            ctx.db.inspects.insert(key, inspect);
        }

        Effect::None
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
                Err(err) => Some(NoteErr { text: err }.into()),
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RefreshProjectInfo {
    pub project_name: String,
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

            match result {
                Ok(project_info) => Some(SetProjectInfo { project_info }.into()),
                Err(err) => Some(NoteErr { text: err }.into()),
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShowInfo(pub Project);

impl Handler for ShowInfo {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.active_view = ViewId::Info;
        ctx.db.info_scroll = 0;

        let project = self.0;

        Effect::Dispatch(
            RefreshProjectInfo {
                project_name: project.name,
            }
            .into(),
        )
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
pub struct ScrollDownInfo;

impl Handler for ScrollDownInfo {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.info_scroll = ctx.db.info_scroll.saturating_add(1);
        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollUpInfo;

impl Handler for ScrollUpInfo {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.info_scroll = ctx.db.info_scroll.saturating_sub(1);
        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectPrevProject;

impl Handler for SelectPrevProject {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.select_previous_project();
        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectNextProject;

impl Handler for SelectNextProject {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.select_next_project();
        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DockerComposeStop(pub Project);

impl Handler for DockerComposeStop {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        let project = self.0;

        ctx.db.prompt(Prompt::new(
            "Confirm",
            format!(
                "Are you sure that you want to stop the project '{}'?",
                project.name
            ),
            docker_compose_action(project, PendingOperation::Stopping, |_| {
                args(["compose", "stop"])
            }),
        ));

        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DockerComposeStart(pub Project);

impl Handler for DockerComposeStart {
    fn handle(self, _ctx: &mut Ctx) -> Effect<Message> {
        let project = self.0;
        docker_compose_action(project, PendingOperation::Starting, |_| {
            args(["compose", "start"])
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DockerComposeUp(pub Project);

impl Handler for DockerComposeUp {
    fn handle(self, _ctx: &mut Ctx) -> Effect<Message> {
        let project = self.0;

        docker_compose_action(project, PendingOperation::Starting, |_| {
            args(["compose", "up", "-d"])
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DockerComposeDown(pub Project);

impl Handler for DockerComposeDown {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        let project = self.0;

        ctx.db.prompt(Prompt::new(
            "Confirm",
            format!(
                "Are you sure that you want to down the project '{}'?",
                project.name
            ),
            docker_compose_action(project, PendingOperation::Starting, |_| {
                args(["compose", "down"])
            }),
        ));

        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DockerComposeRestart(pub Project);

impl Handler for DockerComposeRestart {
    fn handle(self, _ctx: &mut Ctx) -> Effect<Message> {
        let project = self.0;

        docker_compose_action(project, PendingOperation::Starting, |_| {
            args(["compose", "restart"])
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectProject(pub Project);

impl Handler for SelectProject {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        let project = self.0;

        ctx.db.select_project(project);
        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeselectProject;

impl Handler for DeselectProject {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.active_context = ContextId::Projects;
        ctx.db.active_container_index = None;

        Effect::None
    }
}
