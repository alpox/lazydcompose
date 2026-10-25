use color_eyre::eyre::Context;
use futures::try_join;
use ratatui::style::{Color, Style};

use crate::{
    cli::CliCommand,
    data::{compose::Project, inspect::ContainerInspect},
    effect::Effect,
    events::general::Batch,
    model::{ContextId, Note, PendingOperation, Prompt, ResourceId, ViewId},
    system::{Ctx, Handler, Message},
    util::{ResultExt, args},
};

fn note_styled(ctx: &mut Ctx, text: impl Into<String>, style: Style) -> Effect<Message> {
    ctx.db.notes.push(Note::new(text.into()).style(style));
    Effect::None
}

fn note_err(ctx: &mut Ctx, text: impl Into<String>) -> Effect<Message> {
    note_styled(ctx, text, Style::new().fg(Color::Red))
}

async fn fetch_projects() -> Result<Vec<Project>, String> {
    let projects = CliCommand::docker(["compose", "ls", "-a", "--format", "json"]);
    let containers = CliCommand::docker(["container", "ls", "-a", "--format", "json"]);

    try_join!(projects.run(), containers.run())
        .and_then(|(ls, containers)| Project::parse(ls, containers))
        .stringify_err()
}

pub fn pending_action(
    command: CliCommand,
    resource_ids: Vec<ResourceId>,
    op: PendingOperation,
) -> Effect<Message> {
    let mut effects: Vec<_> = resource_ids
        .iter()
        .map(|resource_id| {
            Effect::Dispatch(
                InitPendingAction {
                    resource_id: resource_id.clone(),
                    pending_op: op.clone(),
                }
                .into(),
            )
        })
        .collect();

    effects.push(Effect::perform(async move {
        let mut messages: Vec<Message> = Vec::new();

        if let Err(text) = command.run().await.stringify_err() {
            messages.push(NoteErr { text }.into());
        }
        if let Ok(projects) = fetch_projects().await {
            messages.push(SetProjects { projects }.into());
        }
        messages.push(StopPendingActions { resource_ids }.into());

        Some(Batch(messages).into())
    }));

    Effect::Batch(effects)
}

pub fn docker_compose_action(
    project: &Project,
    op: PendingOperation,
    args: Vec<String>,
) -> Effect<Message> {
    let command = CliCommand::docker(args).cwd(project.folder());
    let resource_ids = project
        .containers
        .iter()
        .map(|container| ResourceId::Container(container.id.clone()))
        .collect();

    pending_action(command, resource_ids, op)
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
pub struct SetProjects {
    projects: Vec<Project>,
}

impl Handler for SetProjects {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.projects = self.projects;
        if !ctx.db.projects.is_empty() && ctx.db.active_project_index.is_none() {
            ctx.db.active_project_index = Some(0);
        }
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
            match fetch_projects().await {
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
pub struct ShowInfo {
    pub project_name: String,
}

impl Handler for ShowInfo {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.active_view = ViewId::Info;
        ctx.db.info_scroll = 0;

        Effect::Dispatch(
            RefreshProjectInfo {
                project_name: self.project_name,
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
            docker_compose_action(
                &project,
                PendingOperation::Stopping,
                args(["compose", "stop"]),
            ),
        ));

        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DockerComposeStart(pub Project);

impl Handler for DockerComposeStart {
    fn handle(self, _ctx: &mut Ctx) -> Effect<Message> {
        let project = self.0;
        docker_compose_action(
            &project,
            PendingOperation::Starting,
            args(["compose", "start"]),
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DockerComposeUp(pub Project);

impl Handler for DockerComposeUp {
    fn handle(self, _ctx: &mut Ctx) -> Effect<Message> {
        let project = self.0;

        docker_compose_action(
            &project,
            PendingOperation::Starting,
            args(["compose", "up", "-d"]),
        )
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
            docker_compose_action(
                &project,
                PendingOperation::Stopping,
                args(["compose", "down"]),
            ),
        ));

        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DockerComposeRestart(pub Project);

impl Handler for DockerComposeRestart {
    fn handle(self, _ctx: &mut Ctx) -> Effect<Message> {
        let project = self.0;

        docker_compose_action(
            &project,
            PendingOperation::Restarting,
            args(["compose", "restart"]),
        )
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

#[derive(Clone, Debug, PartialEq)]
pub struct StopPendingActions {
    pub resource_ids: Vec<ResourceId>,
}

impl Handler for StopPendingActions {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        for resource_id in &self.resource_ids {
            ctx.db.stop_pending_action(resource_id);
        }

        Effect::None
    }
}
