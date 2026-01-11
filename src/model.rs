use std::{collections::HashMap, time::Duration};

use itertools::Itertools;
use ratatui::style::Style;
use tokio::time::Instant;

use crate::{
    cli::{Container, Project, State},
    cmd::{BoxedCmd, map_cmd},
    util::wrap_around_optional,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FocusLevel {
    #[default]
    Project,
    Container,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ContextId {
    #[default]
    Projects,
    Containers,
    Logs,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayContextId {
    BindingsPopup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PanelSize {
    #[default]
    Normal,
    Expanded,
    Collapsed,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum RunningState {
    #[default]
    Running,
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceId {
    Container(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingOperation {
    Starting,
    Stopping,
    Restarting,
}

trait OperationComplete {
    fn is_complete(&self, op: &PendingOperation) -> bool;
}

impl OperationComplete for Container {
    fn is_complete(&self, op: &PendingOperation) -> bool {
        match op {
            PendingOperation::Starting => self.state == State::Running,
            PendingOperation::Stopping => self.state == State::Exited,
            PendingOperation::Restarting => self.state == State::Running,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note {
    pub timestamp: Instant,
    pub duration: Duration,
    pub text: String,
    pub style: Style,
}

impl Default for Note {
    fn default() -> Self {
        Self {
            timestamp: Instant::now(),
            duration: Duration::from_secs(3),
            text: Default::default(),
            style: Default::default(),
        }
    }
}

impl Note {
    pub fn new(text: String) -> Self {
        Self {
            timestamp: Instant::now(),
            duration: Duration::from_secs(3),
            style: Default::default(),
            text,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn finished(&self) -> bool {
        self.timestamp.elapsed() >= self.duration
    }
}

#[derive(Clone, Debug)]
pub struct Model {
    // Application data
    pub running_state: RunningState,
    pub projects: Vec<Project>,
    pub pending_operations: HashMap<ResourceId, PendingOperation>,

    // UI state
    pub focus: FocusLevel,
    pub active_context: ContextId,
    pub active_project_index: Option<usize>,
    pub active_container_index: Option<usize>,

    pub active_overlay_context: Option<OverlayContextId>,

    pub notes: Vec<Note>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            running_state: RunningState::Running,
            projects: vec![],
            pending_operations: Default::default(),

            focus: FocusLevel::default(),
            active_context: ContextId::default(),
            active_project_index: None,
            active_container_index: None,

            active_overlay_context: None,

            notes: vec![],
        }
    }
}

impl Model {
    pub fn selected_project(&self) -> Option<Project> {
        self.active_project_index
            .and_then(|index| self.projects.get(index))
            .cloned()
    }

    pub fn selected_project_containers(&self) -> Option<Vec<Container>> {
        Some(self.selected_project()?.containers)
    }

    pub fn selected_container(&self) -> Option<Container> {
        self.active_container_index.and_then(|index| {
            self.selected_project_containers()
                .and_then(|cs| cs.get(index).cloned())
        })
    }

    pub fn num_containers(&self) -> usize {
        self.projects
            .iter()
            .map(|project| project.containers.len())
            .sum()
    }

    pub fn project_index_for_container(&self, container_index: usize) -> Option<usize> {
        self.projects
            .iter()
            .map(|project| project.containers.len())
            .scan(0_usize, |state, num_containers| {
                *state = state.saturating_add(num_containers);
                Some(*state)
            })
            .find_position(|num_containers| container_index < *num_containers)
            .map(|(idx, _)| idx)
    }

    fn correct_project_index(&mut self) {
        let corrected_project_index = self
            .active_container_index
            .and_then(|idx| self.project_index_for_container(idx));

        self.active_project_index = corrected_project_index;
    }

    pub fn select_project(&mut self, project_index: Option<usize>) {
        if project_index.is_none() {
            self.active_context = ContextId::Projects;
            self.active_container_index = None;
            self.active_project_index = None;

            return;
        }

        self.active_context = ContextId::Containers;
        self.active_container_index = self
            .selected_project()
            .filter(|project| !project.containers.is_empty())
            .and(project_index)
            .map(|idx| self.project_container_offset(idx));

        self.correct_project_index();
    }

    pub fn select_previous_container(&mut self) {
        self.active_container_index = wrap_around_optional(
            self.active_container_index,
            -1,
            self.num_containers().saturating_sub(1),
        );
        self.correct_project_index();
    }

    pub fn select_next_container(&mut self) {
        self.active_container_index = wrap_around_optional(
            self.active_container_index,
            1,
            self.num_containers().saturating_sub(1),
        );
        self.correct_project_index();
    }

    pub fn project_container_offset(&self, project_index: usize) -> usize {
        self.projects
            .iter()
            .take(project_index)
            .map(|project| project.containers.len())
            .sum()
    }

    pub fn init_pending_action(&mut self, resource_id: ResourceId, op: PendingOperation) {
        self.pending_operations.insert(resource_id, op);
    }

    pub fn stop_pending_action(&mut self, resource_id: &ResourceId) {
        self.pending_operations.remove(resource_id);
    }

    pub fn container_by_id(&self, id: &str) -> Option<Container> {
        self.projects.iter().find_map(|project| {
            project
                .containers
                .iter()
                .find(|container| container.id == id)
                .cloned()
        })
    }

    pub fn update_pending_actions(&mut self) {
        let to_remove: Vec<_> = self
            .pending_operations
            .iter()
            .filter(|(resource_id, op)| match resource_id {
                ResourceId::Container(id) => self
                    .container_by_id(id)
                    .map(|container| container.is_complete(op))
                    .unwrap_or(false),
            })
            .map(|(k, _)| k.clone())
            .collect();

        for id in to_remove {
            self.pending_operations.remove(&id);
        }
    }

    pub fn has_pending_action(&self, resource_id: &ResourceId) -> bool {
        self.pending_operations.contains_key(resource_id)
    }

    pub fn project_container_index(
        &self,
        project_index: usize,
        container_index: Option<usize>,
    ) -> Option<usize> {
        container_index.map(|idx| idx.saturating_sub(self.project_container_offset(project_index)))
    }
}

pub enum Action<Msg> {
    None,
    Cmd(BoxedCmd<Msg>),
    BlockingCmd(BoxedCmd<Msg>),
}

impl<Msg> Action<Msg> {
    pub fn map<F, NewMsg>(self, f: F) -> Action<NewMsg>
    where
        Msg: Send + 'static,
        NewMsg: Send + 'static,
        F: Fn(Msg) -> NewMsg + Clone + Send + Sync + 'static,
    {
        match self {
            Action::None => Action::None,
            Action::Cmd(cmd) => Action::Cmd(Box::new(map_cmd(cmd, f))),
            Action::BlockingCmd(cmd) => Action::BlockingCmd(Box::new(map_cmd(cmd, f))),
        }
    }
}

pub struct ChildAction<Msg, OutMsg>(pub Action<Msg>, pub Option<OutMsg>);

pub trait ChildActionAdaptor<Model, Msg> {
    fn adapt(self, model: &mut Model) -> Action<Msg>;
}

impl<Msg, OutMsg> ChildAction<Msg, OutMsg> {
    pub fn none() -> Self {
        ChildAction(Action::None, None)
    }

    pub fn out(out: OutMsg) -> Self {
        ChildAction(Action::None, Some(out))
    }

    pub fn new(result: Action<Msg>) -> Self {
        ChildAction(result, None)
    }

    pub fn map_msg<F, NewMsg>(self, f: F) -> ChildAction<NewMsg, OutMsg>
    where
        Msg: Send + 'static,
        NewMsg: Send + 'static,
        F: Fn(Msg) -> NewMsg + Clone + Send + Sync + 'static,
    {
        ChildAction(self.0.map(f), self.1)
    }

    pub fn handle_out<F>(self, handler: F) -> Action<Msg>
    where
        F: FnOnce(OutMsg) -> Action<Msg>,
    {
        match self.1 {
            Some(out) => handler(out),
            None => self.0,
        }
    }

    pub fn into_inner(self) -> Action<Msg> {
        self.0
    }
}
