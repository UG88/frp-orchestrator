use anyhow::Result;
use clap::Parser;
use frp_gateway::firewall::FirewallManager;
use frp_gateway::frps_config::FrpsConfigGenerator;
use frp_gateway::health::GatewayTelemetry;
use frp_shared::config::GatewayConfig;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(name = "frp-gateway", about = "FRP Gateway Node Management Daemon")]
struct Args {
    #[arg(short, long, default_value = "gateway.toml")]
    config: PathBuf,

    #[arg(long, default_value = "http://127.0.0.1:8080")]
    controller_url: String,

    #[arg(long, default_value = "")]
    controller_token: String,

    #[arg(long)]
    apply_firewall: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,frp_gateway=debug,frp_shared=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();
    info!(config_path = %args.config.display(), "Starting FRP Gateway Daemon");

    let config = if args.config.exists() {
        let content = std::fs::read_to_string(&args.config)?;
        let cfg: GatewayConfig = toml::from_str(&content)?;
        cfg.validate()?;
        cfg
    } else {
        info!("Gateway config not found, creating template gateway.toml");
        let default_cfg = GatewayConfig {
            id: "gw-sg-01".to_string(),
            region: "singapore".to_string(),
            public_ip: "198.51.100.10".to_string(),
            control_port: 7000,
            tcp_port_range: frp_shared::config::PortRangeConfig {
                start: 30000,
                end: 40000,
            },
            udp_port_range: frp_shared::config::PortRangeConfig {
                start: 30000,
                end: 40000,
            },
            reserved_ports: vec![30000, 30001],
            token: "ENV:FRP_GATEWAY_TOKEN".to_string(),
            tls_enable: true,
            dashboard_port: Some(7500),
        };
        let toml_str = toml::to_string_pretty(&default_cfg)?;
        std::fs::write(&args.config, toml_str)?;
        default_cfg
    };

    // Write frps.toml
    let frps_path = PathBuf::from("/opt/frp/frps.toml");
    let target = if cfg!(windows) {
        PathBuf::from("frps.toml")
    } else {
        frps_path
    };

    FrpsConfigGenerator::write_config(&config, &target)?;

    if args.apply_firewall {
        info!("Applying firewall rules...");
        FirewallManager::apply_ufw(&config).await?;
    }

    let controller_token = frp_shared::config::resolve_secret(&args.controller_token);
    let telemetry = GatewayTelemetry::new(
        config.id.clone(),
        args.controller_url,
        controller_token,
        config.dashboard_port.unwrap_or(7500),
    );

    telemetry.run(15).await?;

    Ok(())
}
