use crate::api::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use frp_shared::api_types::{HeartbeatNodeRequest, RegisterNodeRequest, RegisterNodeResponse};
use frp_shared::models::Node;
use serde_json::json;

pub async fn list_nodes(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.list_nodes() {
        Ok(nodes) => (StatusCode::OK, Json(json!({ "nodes": nodes }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn register_node(
    State(state): State<AppState>,
    Json(payload): Json<RegisterNodeRequest>,
) -> impl IntoResponse {
    // Verify gateway exists
    let gateway = match state.db.get_gateway(&payload.assigned_gateway_id) {
        Ok(Some(gw)) => gw,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("Assigned gateway {} does not exist", payload.assigned_gateway_id) })),
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

    let node = Node {
        id: payload.node_id,
        name: payload.name,
        pterodactyl_node_id: payload.pterodactyl_node_id,
        assigned_gateway_id: payload.assigned_gateway_id,
        local_ip: payload.local_ip,
        is_healthy: true,
        last_heartbeat: Some(Utc::now()),
        agent_token: payload.agent_token,
        created_at: Utc::now(),
    };

    match state.db.upsert_node(&node) {
        Ok(_) => {
            let resp = RegisterNodeResponse {
                node,
                assigned_gateway: gateway,
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn node_heartbeat(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<HeartbeatNodeRequest>,
) -> impl IntoResponse {
    match state.db.update_node_heartbeat(&id, payload.is_healthy) {
        Ok(_) => (StatusCode::OK, Json(json!({ "status": "ok" }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
