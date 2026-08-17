use anyhow::{Context, Result};
use clap::Parser;
use frp_agent::runner::AgentRunner;
use frp_shared::config::AgentConfig;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(name = "frp-agent", about = "FRP Node Agent for Pterodactyl Nodes")]
struct Args {
    #[arg(short, long, default_value = "agent.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,frp_agent=debug,frp_shared=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();
    info!(config_path = %args.config.display(), "Starting FRP Agent Daemon");

    let config = if args.config.exists() {
        AgentConfig::load_from_file(&args.config)
            .context("Failed to load agent configuration file")?
    } else {
        info!("Agent config not found, generating template agent.toml");
        let default_cfg = AgentConfig {
            agent_id: "agent-sg-node-01".to_string(),
            controller_url: "http://127.0.0.1:8080".to_string(),
            agent_token: "ENV:FRP_AGENT_TOKEN".to_string(),
            pterodactyl_node_id: 1,
            frpc_binary_path: "/opt/frp/frpc".to_string(),
            frpc_config_dir: "/opt/frp/conf.d".to_string(),
            frpc_main_config: "/opt/frp/frpc.toml".to_string(),
            frpc_admin_addr: "127.0.0.1:7400".to_string(),
            frpc_admin_user: "admin".to_string(),
            frpc_admin_password: "admin".to_string(),
            heartbeat_interval_secs: 15,
        };
        let toml_str = toml::to_string_pretty(&default_cfg)?;
        std::fs::write(&args.config, toml_str)?;
        default_cfg
    };

    let runner = AgentRunner::new(config);
    runner.run().await?;

    Ok(())
}
