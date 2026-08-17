use anyhow::{Context, Result};
use clap::Parser;
use frp_controller::allocation_manager::AllocationManager;
use frp_controller::api::{create_router, AppState};
use frp_controller::db::Database;
use frp_controller::port_manager::PortManager;
use frp_controller::protocol_detector::ProtocolDetector;
use frp_controller::pterodactyl_client::PterodactylClient;
use frp_controller::reconciler::Reconciler;
use frp_shared::config::ControllerConfig;
use frp_shared::dns::{CloudflareProvider, DnsProvider, ManualProvider};
use frp_shared::models::Gateway;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(name = "frp-controller", about = "FRP Gateway Central Orchestrator")]
struct Args {
    #[arg(short, long, default_value = "controller.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,frp_controller=debug,frp_shared=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();
    info!(config_path = %args.config.display(), "Starting FRP Orchestrator Controller");

    let config = if args.config.exists() {
        ControllerConfig::load_from_file(&args.config)
            .context("Failed to load controller configuration file")?
    } else {
        info!("Configuration file not found, creating default controller.toml");
        let default_cfg = ControllerConfig {
            listen_addr: "0.0.0.0:8080".to_string(),
            api_key: "ENV:CONTROLLER_API_KEY".to_string(),
            database_path: "controller.db".to_string(),
            reconciliation_interval_secs: 30,
            pterodactyl: frp_shared::config::PterodactylConfig {
                url: "https://panel.example.com".to_string(),
                api_key: "ENV:PTERODACTYL_API_KEY".to_string(),
                sync_interval_secs: 30,
                auto_expose_default: true,
                egg_protocol_overrides: std::collections::HashMap::new(),
            },
            dns: Some(frp_shared::config::DnsConfig {
                default_domain: Some("mc.example.com".to_string()),
                cloudflare: Some(frp_shared::config::CloudflareConfig {
                    enabled: false,
                    api_token: "ENV:CLOUDFLARE_API_TOKEN".to_string(),
                    zone_id: "your_zone_id_here".to_string(),
                    proxied: false,
                    ttl: 1,
                }),
                route53: None,
            }),
            gateways: vec![frp_shared::config::GatewayConfig {
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
            }],
        };
        let toml_str = toml::to_string_pretty(&default_cfg)?;
        std::fs::write(&args.config, toml_str)?;
        default_cfg
    };

    let db = Database::open(&config.database_path)
        .context(format!("Failed to open database at {}", config.database_path))?;

    // Seed/sync gateways from configuration
    for gw_cfg in &config.gateways {
        let token = frp_shared::config::resolve_secret(&gw_cfg.token);
        let gw = Gateway {
            id: gw_cfg.id.clone(),
            region: gw_cfg.region.clone(),
            public_ip: gw_cfg.public_ip.clone(),
            control_port: gw_cfg.control_port,
            tcp_port_range_start: gw_cfg.tcp_port_range.start,
            tcp_port_range_end: gw_cfg.tcp_port_range.end,
            udp_port_range_start: gw_cfg.udp_port_range.start,
            udp_port_range_end: gw_cfg.udp_port_range.end,
            reserved_ports: gw_cfg.reserved_ports.clone(),
            is_healthy: true,
            last_heartbeat: Some(chrono::Utc::now()),
            token,
            created_at: chrono::Utc::now(),
        };
        db.upsert_gateway(&gw)?;
    }

    let port_mgr = PortManager::new(db.clone());
    let proto_detector = Arc::new(ProtocolDetector::new(
        config.pterodactyl.egg_protocol_overrides.clone(),
    ));

    // DNS Provider resolution (Cloudflare disabled by default; manual fallback)
    let dns_provider: Arc<dyn DnsProvider> = if let Some(dns_cfg) = &config.dns {
        if let Some(cf) = &dns_cfg.cloudflare {
            if cf.enabled {
                info!("Initializing Cloudflare DNS provider");
                Arc::new(CloudflareProvider::new(
                    cf.resolved_api_token(),
                    cf.zone_id.clone(),
                    cf.proxied,
                    cf.ttl,
                ))
            } else {
                info!("Cloudflare DNS is disabled by default; using manual provider");
                Arc::new(ManualProvider)
            }
        } else {
            Arc::new(ManualProvider)
        }
    } else {
        Arc::new(ManualProvider)
    };

    let default_domain = config.dns.as_ref().and_then(|d| d.default_domain.clone());
    let allocation_mgr = AllocationManager::new(
        db.clone(),
        port_mgr.clone(),
        proto_detector,
        dns_provider,
        default_domain,
    );

    let ptero_client = PterodactylClient::new(
        config.pterodactyl.url.clone(),
        config.pterodactyl.resolved_api_key(),
    );

    let reconciler = Reconciler::new(db.clone(), ptero_client, allocation_mgr.clone());

    // Spawn background reconciliation task
    let reconciler_bg = reconciler.clone();
    let recon_interval = config.reconciliation_interval_secs;
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(recon_interval));
        loop {
            interval.tick().await;
            info!("Running periodic state reconciliation...");
            let res = reconciler_bg.reconcile_all().await;
            if !res.errors.is_empty() {
                error!(error_count = res.errors.len(), "Errors encountered during periodic reconciliation");
            }
        }
    });

    let app_state = AppState {
        db,
        port_mgr,
        allocation_mgr,
        reconciler,
        api_key: config.resolved_api_key(),
        start_time: Instant::now(),
    };

    let router = create_router(app_state);

    let listener = tokio::net::TcpListener::bind(&config.listen_addr)
        .await
        .context(format!("Failed to bind to {}", config.listen_addr))?;

    info!(listen_addr = %config.listen_addr, "Controller HTTP server is listening");

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Controller server encountered an error")?;

    info!("Controller shut down gracefully");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, initiating graceful shutdown");
        },
        _ = terminate => {
            info!("Received SIGTERM, initiating graceful shutdown");
        },
    }
}
