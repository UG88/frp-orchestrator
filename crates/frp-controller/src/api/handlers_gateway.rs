use crate::api::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use frp_shared::models::Gateway;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct CreateGatewayPayload {
    pub id: String,
    pub region: String,
    pub public_ip: String,
    #[serde(default = "default_7000")]
    pub control_port: u16,
    pub tcp_start: u16,
    pub tcp_end: u16,
    pub udp_start: u16,
    pub udp_end: u16,
    #[serde(default)]
    pub reserved_ports: Vec<u16>,
    pub token: String,
}

fn default_7000() -> u16 {
    7000
}

pub async fn list_gateways(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.list_gateways() {
        Ok(gateways) => (StatusCode::OK, Json(json!({ "gateways": gateways }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn create_gateway(
    State(state): State<AppState>,
    Json(payload): Json<CreateGatewayPayload>,
) -> impl IntoResponse {
    if payload.tcp_start > payload.tcp_end || payload.udp_start > payload.udp_end {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Start port must be less than or equal to end port" })),
        )
            .into_response();
    }

    let gw = Gateway {
        id: payload.id,
        region: payload.region,
        public_ip: payload.public_ip,
        control_port: payload.control_port,
        tcp_port_range_start: payload.tcp_start,
        tcp_port_range_end: payload.tcp_end,
        udp_port_range_start: payload.udp_start,
        udp_port_range_end: payload.udp_end,
        reserved_ports: payload.reserved_ports,
        is_healthy: true,
        last_heartbeat: Some(Utc::now()),
        token: payload.token,
        created_at: Utc::now(),
    };

    match state.db.upsert_gateway(&gw) {
        Ok(_) => (StatusCode::CREATED, Json(json!({ "gateway": gw }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn get_gateway_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.db.get_gateway(&id) {
        Ok(Some(gw)) => match state.port_mgr.get_pool_status(&gw) {
            Ok(pool) => (StatusCode::OK, Json(json!({ "gateway": gw, "ports": pool }))).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response(),
        },
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Gateway not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct GatewayHeartbeatPayload {
    pub is_healthy: bool,
}

pub async fn gateway_heartbeat(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<GatewayHeartbeatPayload>,
) -> impl IntoResponse {
    match state.db.update_gateway_heartbeat(&id, payload.is_healthy) {
        Ok(_) => (StatusCode::OK, Json(json!({ "status": "ok" }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
