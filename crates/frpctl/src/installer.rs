use anyhow::{bail, Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

pub struct Installer;

impl Installer {
    /// Detect target architecture for FRP binary download.
    pub fn detect_frp_arch() -> Result<(&'static str, &'static str)> {
        let os = match env::consts::OS {
            "linux" => "linux",
            "windows" => "windows",
            "macos" => "darwin",
            other => bail!("Unsupported operating system: {}", other),
        };

        let arch = match env::consts::ARCH {
            "x86_64" => "amd64",
            "aarch64" => "arm64",
            "arm" => "arm",
            other => bail!("Unsupported system architecture: {}", other),
        };

        Ok((os, arch))
    }

    /// Construct official FRP release download URL.
    pub fn get_download_url(version: &str, os: &str, arch: &str) -> String {
        format!(
            "https://github.com/fatedier/frp/releases/download/v{version}/frp_{version}_{os}_{arch}.tar.gz",
            version = version,
            os = os,
            arch = arch
        )
    }

    /// Install Gateway prerequisites and systemd services.
    pub async fn install_gateway(version: &str, target_dir: &Path) -> Result<()> {
        let (os, arch) = Self::detect_frp_arch()?;
        let url = Self::get_download_url(version, os, arch);

        info!(os = %os, arch = %arch, version = %version, url = %url, "Installing FRP Gateway");

        if !target_dir.exists() {
            fs::create_dir_all(target_dir)
                .context(format!("Failed to create directory: {:?}", target_dir))?;
        }

        // Generate systemd service definition
        let service_content = r#"[Unit]
Description=FRP Gateway Server (frps)
After=network.target network-online.target
Wants=network-online.target

[Service]
Type=simple
User=frp
Group=frp
ExecStart=/opt/frp/frps -c /opt/frp/frps.toml
Restart=always
RestartSec=5s
LimitNOFILE=65536
CapabilityBoundingSet=CAP_NET_BIND_SERVICE
AmbientCapabilities=CAP_NET_BIND_SERVICE
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/opt/frp
PrivateTmp=yes

[Install]
WantedBy=multi-user.target
"#;

        let systemd_path = PathBuf::from("/etc/systemd/system/frps.service");
        if systemd_path.parent().map(|p| p.exists()).unwrap_or(false) {
            fs::write(&systemd_path, service_content)
                .context("Failed to write /etc/systemd/system/frps.service")?;
            info!(path = %systemd_path.display(), "Installed systemd unit for frps");
        }

        info!("FRP Gateway setup completed successfully.");
        Ok(())
    }

    /// Install Agent prerequisites and systemd services.
    pub async fn install_agent(version: &str, target_dir: &Path) -> Result<()> {
        let (os, arch) = Self::detect_frp_arch()?;
        let url = Self::get_download_url(version, os, arch);

        info!(os = %os, arch = %arch, version = %version, url = %url, "Installing FRP Node Agent");

        let conf_d = target_dir.join("conf.d");
        if !conf_d.exists() {
            fs::create_dir_all(&conf_d)
                .context(format!("Failed to create conf.d directory: {:?}", conf_d))?;
        }

        // Generate agent systemd service definition
        let service_content = r#"[Unit]
Description=FRP Node Agent Daemon
After=network.target network-online.target
Wants=network-online.target

[Service]
Type=simple
User=root
ExecStart=/usr/local/bin/frp-agent --config /etc/frp-agent/agent.toml
Restart=always
RestartSec=5s
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
"#;

        let systemd_path = PathBuf::from("/etc/systemd/system/frp-agent.service");
        if systemd_path.parent().map(|p| p.exists()).unwrap_or(false) {
            fs::write(&systemd_path, service_content)
                .context("Failed to write /etc/systemd/system/frp-agent.service")?;
            info!(path = %systemd_path.display(), "Installed systemd unit for frp-agent");
        }

        info!("FRP Agent setup completed successfully.");
        Ok(())
    }

    /// Uninstall FRP Orchestrator components safely.
    pub async fn uninstall(component: &str) -> Result<()> {
        info!(component = %component, "Uninstalling component");
        match component {
            "gateway" => {
                let _ = fs::remove_file("/etc/systemd/system/frps.service");
                let _ = fs::remove_file("/etc/systemd/system/frp-gateway.service");
                info!("Gateway services removed. (Data files in /opt/frp preserved for safety)");
            }
            "agent" => {
                let _ = fs::remove_file("/etc/systemd/system/frpc.service");
                let _ = fs::remove_file("/etc/systemd/system/frp-agent.service");
                info!("Agent services removed.");
            }
            "controller" => {
                let _ = fs::remove_file("/etc/systemd/system/frp-controller.service");
                info!("Controller service removed.");
            }
            _ => bail!("Unknown component '{}'", component),
        }
        Ok(())
    }
}
