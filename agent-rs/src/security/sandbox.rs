use bollard::container::LogOutput;
use bollard::models::{ContainerCreateBody, HostConfig, Mount, MountTypeEnum};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, LogsOptionsBuilder, RemoveContainerOptionsBuilder,
    WaitContainerOptionsBuilder,
};
use bollard::Docker;
use futures_util::StreamExt;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::security::policy::{SecurityError, validate_command};

const DEFAULT_ENV: &[&str] = &[
    "HOME=/tmp",
    "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
    "USER=helix",
    "LOGNAME=helix",
    "LANG=C.UTF-8",
    "LC_ALL=C.UTF-8",
    "HELIX_SANDBOX=1",
];

const WORKSPACE_MOUNT_TARGET: &str = "/workspace";
const DEFAULT_CONTAINER_USER: &str = "65534:65534";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    pub status_code: i64,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug)]
pub enum SandboxError {
    Docker(bollard::errors::Error),
    InvalidMountDir(PathBuf),
    Policy(SecurityError),
}

impl fmt::Display for SandboxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Docker(err) => write!(f, "docker error: {err}"),
            Self::InvalidMountDir(path) => {
                write!(f, "mount directory is outside the workspace: {}", path.display())
            }
            Self::Policy(err) => write!(f, "policy validation failed: {err}"),
        }
    }
}

impl std::error::Error for SandboxError {}

impl From<bollard::errors::Error> for SandboxError {
    fn from(value: bollard::errors::Error) -> Self {
        Self::Docker(value)
    }
}

impl From<SecurityError> for SandboxError {
    fn from(value: SecurityError) -> Self {
        Self::Policy(value)
    }
}

#[derive(Debug, Clone)]
pub struct DockerSandbox {
    docker: Docker,
    workspace_root: PathBuf,
}

impl DockerSandbox {
    pub fn new(workspace_root: PathBuf) -> Result<Self, SandboxError> {
        Ok(Self {
            docker: Docker::connect_with_local_defaults()?,
            workspace_root: canonicalize_workspace(&workspace_root)?,
        })
    }

    pub fn with_docker(docker: Docker, workspace_root: PathBuf) -> Result<Self, SandboxError> {
        Ok(Self {
            docker,
            workspace_root: canonicalize_workspace(&workspace_root)?,
        })
    }

    pub async fn run_command(
        &self,
        cmd: Vec<String>,
        image: &str,
        mount_dir: PathBuf,
    ) -> Result<Output, SandboxError> {
        let container_name = unique_container_name();
        let mount_dir = self.restrict_mount_dir(&mount_dir)?;
        let command = shell_words::join(cmd.iter().map(String::as_str));
        let validated = validate_command(&command, &self.workspace_root)?;

        let config = self.build_container_config(validated, image, &mount_dir);
        let options = CreateContainerOptionsBuilder::default()
            .name(container_name.as_str())
            .build();

        let response = self.docker.create_container(Some(options), config).await?;
        let container_id = response.id;

        let run_result = async {
            self.docker.start_container(&container_id, None).await?;

            let mut wait_stream = self
                .docker
                .wait_container(
                    &container_id,
                    Some(WaitContainerOptionsBuilder::default().build()),
                );
            let wait_result = wait_stream
                .next()
                .await
                .transpose()?
                .ok_or_else(|| bollard::errors::Error::IOError {
                    err: std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "docker wait stream ended early",
                    ),
                })?;

            let mut logs_stream = self.docker.logs(
                &container_id,
                Some(
                    LogsOptionsBuilder::default()
                        .stdout(true)
                        .stderr(true)
                        .follow(false)
                        .build(),
                ),
            );

            let mut stdout = String::new();
            let mut stderr = String::new();
            while let Some(chunk) = logs_stream.next().await {
                match chunk? {
                    LogOutput::StdOut { message } | LogOutput::Console { message } => {
                        stdout.push_str(&String::from_utf8_lossy(&message));
                    }
                    LogOutput::StdErr { message } => {
                        stderr.push_str(&String::from_utf8_lossy(&message));
                    }
                    LogOutput::StdIn { .. } => {}
                }
            }

            Ok::<Output, SandboxError>(Output {
                status_code: wait_result.status_code,
                stdout,
                stderr,
            })
        }
        .await;

        let remove_options = RemoveContainerOptionsBuilder::default()
            .force(true)
            .build();
        self.docker
            .remove_container(&container_id, Some(remove_options))
            .await?;

        run_result
    }

    fn restrict_mount_dir(&self, mount_dir: &Path) -> Result<PathBuf, SandboxError> {
        let mount_dir = soft_canonicalize::soft_canonicalize(mount_dir)
            .map_err(|_| SandboxError::InvalidMountDir(mount_dir.to_path_buf()))?;
        if !mount_dir.starts_with(&self.workspace_root) {
            return Err(SandboxError::InvalidMountDir(mount_dir));
        }

        Ok(mount_dir)
    }

    fn build_container_config(
        &self,
        cmd: Vec<String>,
        image: &str,
        mount_dir: &Path,
    ) -> ContainerCreateBody {
        ContainerCreateBody {
            image: Some(image.to_string()),
            cmd: Some(cmd),
            working_dir: Some(WORKSPACE_MOUNT_TARGET.to_string()),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            network_disabled: Some(true),
            user: Some(DEFAULT_CONTAINER_USER.to_string()),
            env: Some(DEFAULT_ENV.iter().map(|value| value.to_string()).collect()),
            host_config: Some(HostConfig {
                auto_remove: Some(false),
                network_mode: Some("none".to_string()),
                readonly_rootfs: Some(true),
                mounts: Some(vec![Mount {
                    target: Some(WORKSPACE_MOUNT_TARGET.to_string()),
                    source: Some(mount_dir.display().to_string()),
                    typ: Some(MountTypeEnum::BIND),
                    read_only: Some(false),
                    ..Default::default()
                }]),
                cap_drop: Some(vec!["ALL".to_string()]),
                security_opt: Some(vec!["no-new-privileges:true".to_string()]),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

fn canonicalize_workspace(path: &Path) -> Result<PathBuf, SandboxError> {
    soft_canonicalize::soft_canonicalize(path)
        .map_err(|_| SandboxError::InvalidMountDir(path.to_path_buf()))
}

fn unique_container_name() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("helix-sandbox-{ts}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_workspace() -> PathBuf {
        let root = std::env::temp_dir().join(unique_container_name());
        std::fs::create_dir_all(root.join("project")).unwrap();
        root
    }

    #[test]
    fn container_config_uses_restricted_mounts_and_env() {
        let workspace = temp_workspace();
        let docker = Docker::connect_with_local_defaults().unwrap();
        let sandbox = DockerSandbox::with_docker(docker, workspace.clone()).unwrap();
        let project = workspace.join("project");

        let config = sandbox.build_container_config(
            vec!["git".to_string(), "status".to_string()],
            "alpine:3.20",
            &project,
        );

        assert_eq!(config.image.as_deref(), Some("alpine:3.20"));
        assert_eq!(config.user.as_deref(), Some(DEFAULT_CONTAINER_USER));
        assert_eq!(config.working_dir.as_deref(), Some(WORKSPACE_MOUNT_TARGET));
        assert_eq!(config.network_disabled, Some(true));

        let host = config.host_config.expect("host config");
        assert_eq!(host.network_mode.as_deref(), Some("none"));
        assert_eq!(host.readonly_rootfs, Some(true));
        assert_eq!(host.cap_drop, Some(vec!["ALL".to_string()]));

        let mounts = host.mounts.expect("mounts");
        assert_eq!(mounts.len(), 1);
        assert_eq!(mounts[0].target.as_deref(), Some(WORKSPACE_MOUNT_TARGET));
        assert_eq!(mounts[0].source.as_deref(), Some(project.display().to_string().as_str()));
        assert_eq!(mounts[0].typ, Some(MountTypeEnum::BIND));
    }

    #[test]
    fn mount_dir_must_stay_inside_workspace() {
        let workspace = temp_workspace();
        let docker = Docker::connect_with_local_defaults().unwrap();
        let sandbox = DockerSandbox::with_docker(docker, workspace).unwrap();
        let outside = std::env::temp_dir();

        let error = sandbox.restrict_mount_dir(&outside).unwrap_err();
        assert!(matches!(error, SandboxError::InvalidMountDir(_)));
    }
}
