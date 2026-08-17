use crate::doctor::DoctorEngine;
use crate::http_client::ControllerCliClient;
use crate::installer::Installer;
use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use frp_shared::api_types::{CreateMappingRequest, HealthResponse, ReconcileResult};
use frp_shared::models::Protocol;
use serde_json::Value;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "frpctl", about = "CLI management tool for FRP Gateway Orchestrator")]
pub struct Cli {
    #[arg(long, env = "CONTROLLER_URL", default_value = "http://127.0.0.1:8080")]
    pub controller_url: String,

    #[arg(long, env = "CONTROLLER_API_KEY", default_value = "")]
    pub api_key: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Gateway management commands
    Gateway {
        #[command(subcommand)]
        cmd: GatewayCommands,
    },
    /// Node agent management commands
    Agent {
        #[command(subcommand)]
        cmd: AgentCommands,
    },
    /// Allocation inspection and control
    Allocation {
        #[command(subcommand)]
        cmd: AllocationCommands,
    },
    /// FRP mapping inspection and control
    Mapping {
        #[command(subcommand)]
        cmd: MappingCommands,
    },
    /// Port pool inspection
    Port {
        #[command(subcommand)]
        cmd: PortCommands,
    },
    /// Trigger state reconciliation
    Reconcile,
    /// View system health status
    Health,
    /// System and connectivity diagnostics
    Doctor {
        #[arg(long)]
        config: Option<String>,
    },
    /// Interactive configuration wizard
    Init {
        #[arg(value_enum)]
        component: InstallComponent,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Automated installation
    Install {
        #[arg(value_enum)]
        component: InstallComponent,
        #[arg(long, default_value = "0.60.0")]
        version: String,
        #[arg(long, default_value = "/opt/frp")]
        dir: PathBuf,
    },
    /// Remove installed components
    Uninstall {
        #[arg(value_enum)]
        component: InstallComponent,
    },
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstallComponent {
    Gateway,
    Agent,
    Controller,
}

#[derive(Subcommand, Debug)]
pub enum GatewayCommands {
    /// List all registered FRP gateways
    List,
    /// View detailed status of a gateway
    Status { id: String },
}

#[derive(Subcommand, Debug)]
pub enum AgentCommands {
    /// List all registered Pterodactyl node agents
    List,
    /// View status of an agent
    Status { id: String },
}

#[derive(Subcommand, Debug)]
pub enum AllocationCommands {
    /// List all tracked Pterodactyl allocations
    List,
    /// Show details for a specific allocation
    Show { id: String },
}

#[derive(Subcommand, Debug)]
pub enum MappingCommands {
    /// List all active FRP proxy mappings
    List,
    /// Manually create a proxy mapping
    Create(CreateMappingArgs),
    /// Delete a proxy mapping
    Delete { id: String },
}

#[derive(Args, Debug)]
pub struct CreateMappingArgs {
    #[arg(long)]
    pub allocation_id: String,
    #[arg(long)]
    pub node_id: String,
    #[arg(long)]
    pub local_ip: String,
    #[arg(long)]
    pub local_port: u16,
    #[arg(long, default_value = "auto")]
    pub protocol: String,
    #[arg(long)]
    pub server_id: Option<String>,
    #[arg(long)]
    pub server_name: Option<String>,
    #[arg(long)]
    pub alias: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum PortCommands {
    /// View available and allocated ports per gateway
    List,
}

pub async fn run_cli(cli: Cli) -> Result<()> {
    let client = ControllerCliClient::new(cli.controller_url.clone(), cli.api_key.clone());

    match cli.command {
        Commands::Gateway { cmd } => match cmd {
            GatewayCommands::List => {
                let resp: Value = client.get("/api/v1/gateways").await?;
                println!("{}", serde_json::to_string_pretty(&resp)?);
            }
            GatewayCommands::Status { id } => {
                let resp: Value = client.get(&format!("/api/v1/gateways/{}/status", id)).await?;
                println!("{}", serde_json::to_string_pretty(&resp)?);
            }
        },
        Commands::Agent { cmd } => match cmd {
            AgentCommands::List => {
                let resp: Value = client.get("/api/v1/nodes").await?;
                println!("{}", serde_json::to_string_pretty(&resp)?);
            }
            AgentCommands::Status { id } => {
                let resp: Value = client.get(&format!("/api/v1/agent/{}/desired-state", id)).await?;
                println!("{}", serde_json::to_string_pretty(&resp)?);
            }
        },
        Commands::Allocation { cmd } => match cmd {
            AllocationCommands::List => {
                let resp: Value = client.get("/api/v1/allocations").await?;
                println!("{}", serde_json::to_string_pretty(&resp)?);
            }
            AllocationCommands::Show { id } => {
                let resp: Value = client.get(&format!("/api/v1/allocations/{}", id)).await?;
                println!("{}", serde_json::to_string_pretty(&resp)?);
            }
        },
        Commands::Mapping { cmd } => match cmd {
            MappingCommands::List => {
                let resp: Value = client.get("/api/v1/mappings").await?;
                println!("{}", serde_json::to_string_pretty(&resp)?);
            }
            MappingCommands::Create(args) => {
                let protocol = match args.protocol.to_lowercase().as_str() {
                    "tcp" => Protocol::Tcp,
                    "udp" => Protocol::Udp,
                    "both" => Protocol::Both,
                    _ => Protocol::Auto,
                };
                let payload = CreateMappingRequest {
                    allocation_id: args.allocation_id,
                    node_id: args.node_id,
                    server_id: args.server_id,
                    server_name: args.server_name,
                    local_ip: args.local_ip,
                    local_port: args.local_port,
                    protocol,
                    custom_alias: args.alias,
                    gateway_id: None,
                };
                let resp: Value = client.post("/api/v1/mappings", &payload).await?;
                println!("{}", serde_json::to_string_pretty(&resp)?);
            }
            MappingCommands::Delete { id } => {
                let resp: Value = client.delete(&format!("/api/v1/mappings/{}", id)).await?;
                println!("{}", serde_json::to_string_pretty(&resp)?);
            }
        },
        Commands::Port { cmd } => match cmd {
            PortCommands::List => {
                let resp: Value = client.get("/api/v1/ports/status").await?;
                println!("{}", serde_json::to_string_pretty(&resp)?);
            }
        },
        Commands::Reconcile => {
            println!("Triggering state reconciliation...");
            let resp: ReconcileResult = client.post("/api/v1/reconcile", &serde_json::json!({})).await?;
            println!("Reconciliation complete:");
            println!("  Created mappings: {}", resp.created_mappings);
            println!("  Removed mappings: {}", resp.removed_mappings);
            println!("  Updated mappings: {}", resp.updated_mappings);
            if !resp.errors.is_empty() {
                println!("  Errors ({}):", resp.errors.len());
                for e in resp.errors {
                    println!("    - {}", e);
                }
            }
        }
        Commands::Health => {
            let resp: HealthResponse = client.get("/health").await?;
            println!("FRP Gateway Orchestrator Health Report");
            println!("======================================");
            println!("Status:           {}", resp.status);
            println!("Version:          {}", resp.version);
            println!("Database:         {}", if resp.database_ok { "OK" } else { "ERROR" });
            println!("Gateways:         {}/{} online", resp.healthy_gateways, resp.total_gateways);
            println!("Nodes:            {}/{} healthy", resp.healthy_nodes, resp.total_nodes);
            println!("Active Mappings:  {}", resp.active_mappings);
            println!("Uptime:           {} seconds", resp.uptime_secs);
        }
        Commands::Doctor { config } => {
            let doctor = DoctorEngine::new(client, config);
            let report = doctor.run_checks().await;
            println!("FRP Orchestrator Doctor Diagnostic");
            println!("==================================");
            for check in &report.checks {
                let icon = if check.passed { "✓" } else { "✗" };
                println!("[{}] {}", icon, check.name);
                println!("    Details: {}", check.details);
                if let Some(ref fix) = check.fix_hint {
                    println!("    Hint:    {}", fix);
                }
                println!();
            }
            if report.overall_status {
                println!("Result: All checks PASSED! System is ready.");
            } else {
                println!("Result: Some checks FAILED. Please review the hints above.");
            }
        }
        Commands::Init { component, output } => match component {
            InstallComponent::Gateway => {
                let path = output.unwrap_or_else(|| PathBuf::from("gateway.toml"));
                crate::wizard::ConfigWizard::setup_gateway(path)?;
            }
            InstallComponent::Agent => {
                let path = output.unwrap_or_else(|| PathBuf::from("agent.toml"));
                crate::wizard::ConfigWizard::setup_agent(path)?;
            }
            InstallComponent::Controller => {
                let path = output.unwrap_or_else(|| PathBuf::from("controller.toml"));
                crate::wizard::ConfigWizard::setup_controller(path)?;
            }
        },
        Commands::Install { component, version, dir } => match component {
            InstallComponent::Gateway => {
                Installer::install_gateway(&version, &dir).await?;
            }
            InstallComponent::Agent => {
                Installer::install_agent(&version, &dir).await?;
            }
            InstallComponent::Controller => {
                println!("Controller setup completed.");
            }
        },
        Commands::Uninstall { component } => match component {
            InstallComponent::Gateway => Installer::uninstall("gateway").await?,
            InstallComponent::Agent => Installer::uninstall("agent").await?,
            InstallComponent::Controller => Installer::uninstall("controller").await?,
        },
    }

    Ok(())
}
