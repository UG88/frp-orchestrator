use crate::api::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use frp_shared::api_types::HealthResponse;
use serde_json::json;

pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let gateways = state.db.list_gateways().unwrap_or_default();
    let healthy_gateways = gateways.iter().filter(|g| g.is_healthy).count();

    let nodes = state.db.list_nodes().unwrap_or_default();
    let healthy_nodes = nodes.iter().filter(|n| n.is_healthy).count();

    let mappings = state.db.list_mappings().unwrap_or_default();
    let active_mappings = mappings.iter().filter(|m| m.is_active).count();

    let uptime_secs = state.start_time.elapsed().as_secs();

    let resp = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database_ok: true,
        total_gateways: gateways.len(),
        healthy_gateways,
        total_nodes: nodes.len(),
        healthy_nodes,
        active_mappings,
        uptime_secs,
    };

    (StatusCode::OK, Json(resp)).into_response()
}

pub async fn trigger_reconciliation(State(state): State<AppState>) -> impl IntoResponse {
    let result = state.reconciler.reconcile_all().await;
    (StatusCode::OK, Json(result)).into_response()
}

pub async fn get_ports_status(State(state): State<AppState>) -> impl IntoResponse {
    let gateways = match state.db.list_gateways() {
        Ok(g) => g,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    let mut statuses = Vec::new();
    for gw in gateways {
        if let Ok(pool) = state.port_mgr.get_pool_status(&gw) {
            statuses.push(pool);
        }
    }

    (StatusCode::OK, Json(json!({ "pools": statuses }))).into_response()
}
