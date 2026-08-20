use crate::api::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use frp_shared::api_types::{AgentDesiredProxy, AgentDesiredStateResponse, AgentReportStateRequest};
use serde_json::json;
use tracing::info;

pub async fn get_desired_state(
    State(state): State<AppState>,
    Path(node_id): Path<String>,
) -> impl IntoResponse {
    let node = match state.db.get_node(&node_id) {
        Ok(Some(n)) => n,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": format!("Node {} not found", node_id) })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    let gateway = match state.db.get_gateway(&node.assigned_gateway_id) {
        Ok(Some(gw)) => gw,
        Ok(None) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Assigned gateway {} not found", node.assigned_gateway_id) })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    let mappings = match state.db.list_mappings_for_node(&node_id) {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    let proxies: Vec<AgentDesiredProxy> = mappings
        .into_iter()
        .map(|m| AgentDesiredProxy {
            proxy_name: format!("mc_{}", m.id.replace('-', "_")),
            mapping_id: m.id,
            allocation_id: m.allocation_id,
            protocol: m.protocol,
            local_ip: m.target_ip,
            local_port: m.target_port,
            remote_port: m.gateway_port,
            gateway_public_ip: gateway.public_ip.clone(),
            gateway_control_port: gateway.control_port,
            gateway_token: gateway.token.clone(),
            fqdn: m.fqdn,
            proxy_protocol_version: None,
        })
        .collect();

    let response = AgentDesiredStateResponse {
        node_id,
        gateway,
        proxies,
    };

    (StatusCode::OK, Json(response)).into_response()
}

pub async fn report_state(
    State(state): State<AppState>,
    Path(node_id): Path<String>,
    Json(payload): Json<AgentReportStateRequest>,
) -> impl IntoResponse {
    info!(
        node_id = %node_id,
        frpc_running = payload.frpc_running,
        active_proxies = payload.running_proxies.len(),
        "Received state report from FRP agent"
    );

    let _ = state.db.update_node_heartbeat(&node_id, payload.frpc_running);

    (StatusCode::OK, Json(json!({ "status": "acknowledged" }))).into_response()
}
