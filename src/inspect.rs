use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};

use crate::cli::State;

fn null_as_default<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Option::<T>::deserialize(d).map(Option::unwrap_or_default)
}

#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct ContainerInspect {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Created")]
    pub created: String,
    #[serde(rename = "Image")]
    pub image_id: String,
    #[serde(rename = "Path")]
    pub path: String,
    #[serde(rename = "Args", default, deserialize_with = "null_as_default")]
    pub args: Vec<String>,
    #[serde(rename = "RestartCount", default)]
    pub restart_count: u32,
    #[serde(rename = "State")]
    pub state: InspectState,
    #[serde(rename = "Config")]
    pub config: InspectConfig,
    #[serde(rename = "HostConfig")]
    pub host_config: InspectHostConfig,
    #[serde(rename = "NetworkSettings")]
    pub network_settings: InspectNetworkSettings,
    #[serde(rename = "Mounts", default, deserialize_with = "null_as_default")]
    pub mounts: Vec<InspectMount>,
}

#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct InspectState {
    #[serde(rename = "Status")]
    pub status: State,
    #[serde(rename = "Running", default)]
    pub running: bool,
    #[serde(rename = "Paused", default)]
    pub paused: bool,
    #[serde(rename = "Restarting", default)]
    pub restarting: bool,
    #[serde(rename = "OOMKilled", default)]
    pub oom_killed: bool,
    #[serde(rename = "Dead", default)]
    pub dead: bool,
    #[serde(rename = "Pid", default)]
    pub pid: i64,
    #[serde(rename = "ExitCode", default)]
    pub exit_code: i32,
    #[serde(rename = "Error", default)]
    pub error: String,
    #[serde(rename = "StartedAt", default)]
    pub started_at: String,
    #[serde(rename = "FinishedAt", default)]
    pub finished_at: String,
    #[serde(rename = "Health")]
    pub health: Option<InspectHealth>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum HealthStatus {
    #[serde(rename = "starting")]
    Starting,
    #[serde(rename = "healthy")]
    Healthy,
    #[serde(rename = "unhealthy")]
    Unhealthy,
    #[serde(rename = "none")]
    None,
}

#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct InspectHealth {
    #[serde(rename = "Status")]
    pub status: HealthStatus,
    #[serde(rename = "FailingStreak", default)]
    pub failing_streak: u32,
    #[serde(rename = "Log", default, deserialize_with = "null_as_default")]
    pub log: Vec<InspectHealthLog>,
}

#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct InspectHealthLog {
    #[serde(rename = "Start")]
    pub start: String,
    #[serde(rename = "End")]
    pub end: String,
    #[serde(rename = "ExitCode")]
    pub exit_code: i32,
    #[serde(rename = "Output", default)]
    pub output: String,
}

#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct InspectConfig {
    #[serde(rename = "Hostname", default)]
    pub hostname: String,
    #[serde(rename = "User", default)]
    pub user: String,
    #[serde(rename = "Env", default, deserialize_with = "null_as_default")]
    pub env: Vec<String>,
    #[serde(rename = "Cmd")]
    pub cmd: Option<Vec<String>>,
    #[serde(rename = "Entrypoint")]
    pub entrypoint: Option<Vec<String>>,
    #[serde(rename = "Image")]
    pub image: String,
    #[serde(rename = "WorkingDir", default)]
    pub working_dir: String,
    #[serde(rename = "Labels", default)]
    pub labels: HashMap<String, String>,
    #[serde(rename = "ExposedPorts", default)]
    pub exposed_ports: HashMap<String, serde_json::Value>,
}

#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct InspectHostConfig {
    #[serde(rename = "RestartPolicy")]
    pub restart_policy: InspectRestartPolicy,
    #[serde(rename = "NetworkMode", default)]
    pub network_mode: String,
    #[serde(rename = "Memory", default)]
    pub memory: i64,
    #[serde(rename = "NanoCpus", default)]
    pub nano_cpus: i64,
    #[serde(rename = "PortBindings", default)]
    pub port_bindings: HashMap<String, Option<Vec<InspectPortBinding>>>,
}

#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct InspectRestartPolicy {
    #[serde(rename = "Name", default)]
    pub name: String,
    #[serde(rename = "MaximumRetryCount", default)]
    pub maximum_retry_count: u32,
}

#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct InspectPortBinding {
    #[serde(rename = "HostIp", default)]
    pub host_ip: String,
    #[serde(rename = "HostPort")]
    pub host_port: String,
}

#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct InspectNetworkSettings {
    #[serde(rename = "Ports", default)]
    pub ports: HashMap<String, Option<Vec<InspectPortBinding>>>,
    #[serde(rename = "Networks", default)]
    pub networks: HashMap<String, InspectNetwork>,
}

#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct InspectNetwork {
    #[serde(rename = "NetworkID", default)]
    pub network_id: String,
    #[serde(rename = "EndpointID", default)]
    pub endpoint_id: String,
    #[serde(rename = "Gateway", default)]
    pub gateway: String,
    #[serde(rename = "IPAddress", default)]
    pub ip_address: String,
    #[serde(rename = "IPPrefixLen", default)]
    pub ip_prefix_len: u8,
    #[serde(rename = "MacAddress", default)]
    pub mac_address: String,
    #[serde(rename = "Aliases", default, deserialize_with = "null_as_default")]
    pub aliases: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum MountKind {
    #[serde(rename = "bind")]
    Bind,
    #[serde(rename = "volume")]
    Volume,
    #[serde(rename = "tmpfs")]
    Tmpfs,
    #[serde(rename = "npipe")]
    NamedPipe,
    #[serde(rename = "cluster")]
    Cluster,
}

#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct InspectMount {
    #[serde(rename = "Type")]
    pub kind: MountKind,
    #[serde(rename = "Name", default)]
    pub name: String,
    #[serde(rename = "Source", default)]
    pub source: String,
    #[serde(rename = "Destination")]
    pub destination: String,
    #[serde(rename = "Driver", default)]
    pub driver: String,
    #[serde(rename = "Mode", default)]
    pub mode: String,
    #[serde(rename = "RW", default)]
    pub rw: bool,
    #[serde(rename = "Propagation", default)]
    pub propagation: String,
}
