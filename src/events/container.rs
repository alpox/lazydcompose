use crate::{
    cli::CliCommand,
    data::compose::{Container, Project},
    effect::Effect,
    events::{
        general::Batch,
        project::{InitPendingAction, NoteErr, UpdatePendingActions},
    },
    model::{PendingOperation, Prompt, ResourceId},
    system::{Ctx, Handler, Message},
    util::args,
};

fn docker_action<F, R>(
    project: Project,
    container: Container,
    op: PendingOperation,
    f: F,
) -> Effect<Message>
where
    F: FnOnce(Project, Container) -> R + Send + 'static,
    R: IntoIterator<Item = String> + Send,
{
    let resource_id = ResourceId::Container(container.id.clone());
    Effect::Batch(vec![
        Effect::Dispatch(
            InitPendingAction {
                resource_id,
                pending_op: op,
            }
            .into(),
        ),
        Effect::perform(async move {
            let args: Vec<_> = f(project.clone(), container.clone()).into_iter().collect();
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
        }),
    ])
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectPrevContainer;

impl Handler for SelectPrevContainer {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.select_previous_container();
        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectNextContainer;

impl Handler for SelectNextContainer {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.select_next_container();
        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DockerContainerStart {
    pub project: Project,
    pub container: Container,
}

impl Handler for DockerContainerStart {
    fn handle(self, _ctx: &mut Ctx) -> Effect<Message> {
        docker_action(
            self.project,
            self.container,
            PendingOperation::Starting,
            |_, container| args(["container", "start", container.names.as_str()]),
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DockerContainerStop {
    pub project: Project,
    pub container: Container,
}

impl Handler for DockerContainerStop {
    fn handle(self, ctx: &mut Ctx) -> Effect<Message> {
        ctx.db.prompt(Prompt::new(
            "Confirm",
            format!(
                "Are you sure that you want to stop the container '{}'?",
                self.container.names
            ),
            docker_action(
                self.project,
                self.container,
                PendingOperation::Stopping,
                |_, container| args(["container", "stop", container.names.as_str()]),
            ),
        ));

        Effect::None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DockerContainerRestart {
    pub project: Project,
    pub container: Container,
}

impl Handler for DockerContainerRestart {
    fn handle(self, _ctx: &mut Ctx) -> Effect<Message> {
        docker_action(
            self.project,
            self.container,
            PendingOperation::Restarting,
            |_, container| args(["container", "restart", container.names.as_str()]),
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DockerContainerFollowLogs {
    pub project: Project,
    pub container: Container,
}

impl Handler for DockerContainerFollowLogs {
    fn handle(self, _ctx: &mut Ctx) -> Effect<Message> {
        Effect::perform_blocking(async move {
            let _ = CliCommand::docker([
                "logs",
                "--follow",
                "--since=60m",
                self.container.id.as_str(),
            ])
            .cwd(self.project.folder())
            .run_tty()
            .await;

            None
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DockerContainerConsole {
    pub project: Project,
    pub container: Container,
}

impl Handler for DockerContainerConsole {
    fn handle(self, _ctx: &mut Ctx) -> Effect<Message> {
        Effect::perform_blocking(async move {
            let _ = CliCommand::docker(["exec", "-it", self.container.id.as_str(), "/bin/sh"])
                .cwd(self.project.folder())
                .run_tty()
                .await;
            None
        })
    }
}
