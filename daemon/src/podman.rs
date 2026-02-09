use std::collections::HashSet;
use std::fmt;

use crate::models::environment::PortMapping;

const DEFAULT_IMAGE: &str = "ubuntu:22.04";

#[derive(Debug)]
pub enum PodmanError {
    NotInstalled,
    CommandFailed {
        command: String,
        stderr: String,
        exit_code: i32,
    },
    ParseError(String),
}

impl fmt::Display for PodmanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PodmanError::NotInstalled => write!(f, "podman is not installed or not in PATH"),
            PodmanError::CommandFailed {
                command,
                stderr,
                exit_code,
            } => write!(
                f,
                "podman command '{}' failed (exit {}): {}",
                command, exit_code, stderr
            ),
            PodmanError::ParseError(msg) => write!(f, "parse error: {}", msg),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PodmanConfig {
    pub podman_path: String,
    pub port_range_start: u16,
    pub port_range_end: u16,
}

impl Default for PodmanConfig {
    fn default() -> Self {
        PodmanConfig {
            podman_path: "podman".to_string(),
            port_range_start: 10000,
            port_range_end: 11000,
        }
    }
}

#[derive(Debug)]
pub struct ExecResult {
    pub output: String,
    pub exit_code: i32,
}

pub async fn check_podman(config: &PodmanConfig) -> Result<String, PodmanError> {
    let output = tokio::process::Command::new(&config.podman_path)
        .arg("--version")
        .output()
        .await
        .map_err(|_| PodmanError::NotInstalled)?;

    if !output.status.success() {
        return Err(PodmanError::NotInstalled);
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(version)
}

pub fn container_name(env_id: &str) -> String {
    let short = &env_id[..env_id.len().min(8)];
    format!("botglue-{}", short)
}

pub async fn create_container(
    config: &PodmanConfig,
    name: &str,
    image: Option<&str>,
    port_bindings: &[PortMapping],
) -> Result<String, PodmanError> {
    let image = image.unwrap_or(DEFAULT_IMAGE);

    let mut args = vec![
        "run".to_string(),
        "-d".to_string(),
        "--name".to_string(),
        name.to_string(),
    ];

    for mapping in port_bindings {
        if let Some(host_port) = mapping.host_port {
            args.push("-p".to_string());
            args.push(format!("{}:{}", host_port, mapping.container_port));
        }
    }

    args.push(image.to_string());
    args.push("sleep".to_string());
    args.push("infinity".to_string());

    let output = tokio::process::Command::new(&config.podman_path)
        .args(&args)
        .output()
        .await
        .map_err(|_| PodmanError::NotInstalled)?;

    if !output.status.success() {
        return Err(PodmanError::CommandFailed {
            command: format!("podman {}", args.join(" ")),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        });
    }

    let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(container_id)
}

pub async fn stop_container(
    config: &PodmanConfig,
    container_id: &str,
) -> Result<(), PodmanError> {
    let output = tokio::process::Command::new(&config.podman_path)
        .args(["stop", container_id])
        .output()
        .await
        .map_err(|_| PodmanError::NotInstalled)?;

    if !output.status.success() {
        return Err(PodmanError::CommandFailed {
            command: format!("podman stop {}", container_id),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        });
    }

    Ok(())
}

pub async fn start_container(
    config: &PodmanConfig,
    container_id: &str,
) -> Result<(), PodmanError> {
    let output = tokio::process::Command::new(&config.podman_path)
        .args(["start", container_id])
        .output()
        .await
        .map_err(|_| PodmanError::NotInstalled)?;

    if !output.status.success() {
        return Err(PodmanError::CommandFailed {
            command: format!("podman start {}", container_id),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        });
    }

    Ok(())
}

pub async fn remove_container(
    config: &PodmanConfig,
    container_id: &str,
) -> Result<(), PodmanError> {
    let output = tokio::process::Command::new(&config.podman_path)
        .args(["rm", "-f", container_id])
        .output()
        .await
        .map_err(|_| PodmanError::NotInstalled)?;

    if !output.status.success() {
        return Err(PodmanError::CommandFailed {
            command: format!("podman rm -f {}", container_id),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        });
    }

    Ok(())
}

pub async fn exec_in_container(
    config: &PodmanConfig,
    container_id: &str,
    command: &str,
) -> Result<ExecResult, PodmanError> {
    let output = tokio::process::Command::new(&config.podman_path)
        .args(["exec", container_id, "sh", "-c", command])
        .output()
        .await
        .map_err(|_| PodmanError::NotInstalled)?;

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let combined = if stderr.is_empty() {
        stdout
    } else if stdout.is_empty() {
        stderr
    } else {
        format!("{}{}", stdout, stderr)
    };

    Ok(ExecResult {
        output: combined,
        exit_code,
    })
}

pub fn allocate_ports(
    config: &PodmanConfig,
    used_ports: &HashSet<u16>,
    requested: &[PortMapping],
) -> Result<Vec<PortMapping>, PodmanError> {
    let mut result = Vec::new();
    let mut newly_used = HashSet::new();

    for mapping in requested {
        if let Some(host_port) = mapping.host_port {
            if used_ports.contains(&host_port) || newly_used.contains(&host_port) {
                return Err(PodmanError::ParseError(format!(
                    "port {} is already in use",
                    host_port
                )));
            }
            newly_used.insert(host_port);
            result.push(mapping.clone());
        } else {
            let mut assigned = None;
            for port in config.port_range_start..config.port_range_end {
                if !used_ports.contains(&port) && !newly_used.contains(&port) {
                    assigned = Some(port);
                    break;
                }
            }
            match assigned {
                Some(port) => {
                    newly_used.insert(port);
                    result.push(PortMapping {
                        name: mapping.name.clone(),
                        container_port: mapping.container_port,
                        host_port: Some(port),
                        protocol: mapping.protocol.clone(),
                    });
                }
                None => {
                    return Err(PodmanError::ParseError(
                        "port range exhausted, no available ports".to_string(),
                    ));
                }
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> PodmanConfig {
        PodmanConfig {
            podman_path: "podman".to_string(),
            port_range_start: 10000,
            port_range_end: 10005,
        }
    }

    #[test]
    fn test_allocate_ports_auto_assign() {
        let config = test_config();
        let used = HashSet::new();
        let requested = vec![
            PortMapping {
                name: "http".to_string(),
                container_port: 8080,
                host_port: None,
                protocol: None,
            },
            PortMapping {
                name: "debug".to_string(),
                container_port: 9229,
                host_port: None,
                protocol: None,
            },
        ];

        let result = allocate_ports(&config, &used, &requested).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].host_port, Some(10000));
        assert_eq!(result[1].host_port, Some(10001));
    }

    #[test]
    fn test_allocate_ports_skip_used() {
        let config = test_config();
        let mut used = HashSet::new();
        used.insert(10000);
        used.insert(10001);

        let requested = vec![PortMapping {
            name: "http".to_string(),
            container_port: 8080,
            host_port: None,
            protocol: None,
        }];

        let result = allocate_ports(&config, &used, &requested).unwrap();
        assert_eq!(result[0].host_port, Some(10002));
    }

    #[test]
    fn test_allocate_ports_explicit_conflict() {
        let config = test_config();
        let mut used = HashSet::new();
        used.insert(10000);

        let requested = vec![PortMapping {
            name: "http".to_string(),
            container_port: 8080,
            host_port: Some(10000),
            protocol: None,
        }];

        let result = allocate_ports(&config, &used, &requested);
        assert!(result.is_err());
    }

    #[test]
    fn test_allocate_ports_range_exhaustion() {
        let config = test_config(); // range 10000..10005 = 5 ports
        let mut used = HashSet::new();
        for p in 10000..10005 {
            used.insert(p);
        }

        let requested = vec![PortMapping {
            name: "http".to_string(),
            container_port: 8080,
            host_port: None,
            protocol: None,
        }];

        let result = allocate_ports(&config, &used, &requested);
        assert!(result.is_err());
    }

    #[test]
    fn test_container_name() {
        assert_eq!(container_name("abcdefgh-1234"), "botglue-abcdefgh");
        assert_eq!(container_name("short"), "botglue-short");
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_container_lifecycle() {
        let config = PodmanConfig::default();

        // Check podman is available
        let version = check_podman(&config).await.expect("podman not installed");
        assert!(!version.is_empty());

        let name = format!("botglue-test-{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
        let ports = vec![PortMapping {
            name: "http".to_string(),
            container_port: 8080,
            host_port: Some(18080),
            protocol: Some("tcp".to_string()),
        }];

        // Create container
        let container_id = create_container(&config, &name, None, &ports)
            .await
            .expect("failed to create container");
        assert!(!container_id.is_empty());

        // Exec in container
        let result = exec_in_container(&config, &container_id, "echo hello")
            .await
            .expect("failed to exec");
        assert_eq!(result.exit_code, 0);
        assert!(result.output.contains("hello"));

        // Stop container
        stop_container(&config, &container_id)
            .await
            .expect("failed to stop");

        // Start container
        start_container(&config, &container_id)
            .await
            .expect("failed to start");

        // Exec again after restart
        let result = exec_in_container(&config, &container_id, "echo world")
            .await
            .expect("failed to exec after restart");
        assert_eq!(result.exit_code, 0);
        assert!(result.output.contains("world"));

        // Remove container
        remove_container(&config, &container_id)
            .await
            .expect("failed to remove");
    }

    #[tokio::test]
    #[ignore]
    async fn test_exec_nonexistent_container() {
        let config = PodmanConfig::default();
        let result = exec_in_container(&config, "nonexistent-container-12345", "echo hello").await;
        // exec should return an ExecResult with non-zero exit code, or the container doesn't exist
        // podman exec on a nonexistent container returns exit code 125
        match result {
            Ok(r) => assert_ne!(r.exit_code, 0),
            Err(_) => {} // also acceptable
        }
    }
}
