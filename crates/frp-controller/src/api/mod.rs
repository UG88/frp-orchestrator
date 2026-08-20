pub mod handlers_agent;
pub mod handlers_gateway;
pub mod handlers_health;
pub mod handlers_mapping;
pub mod handlers_node;

use crate::allocation_manager::AllocationManager;
use crate::db::Database;
use crate::port_manager::PortManager;
use crate::reconciler::Reconciler;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use frp_shared::crypto::constant_time_eq;
use serde_json::json;
use std::time::Instant;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub port_mgr: PortManager,
    pub allocation_mgr: AllocationManager,
    pub reconciler: Reconciler,
    pub api_key: String,
    pub default_proxy_protocol: Option<String>,
    pub start_time: Instant,
}

pub fn create_router(state: AppState) -> Router {
    let auth_state = state.clone();

    let api_routes = Router::new()
        // Gateway routes
        .route("/gateways", get(handlers_gateway::list_gateways).post(handlers_gateway::create_gateway))
        .route("/gateways/:id/status", get(handlers_gateway::get_gateway_status))
        .route("/gateways/:id/heartbeat", post(handlers_gateway::gateway_heartbeat))
        // Node routes
        .route("/nodes", get(handlers_node::list_nodes))
        .route("/nodes/register", post(handlers_node::register_node))
        .route("/nodes/:id/heartbeat", post(handlers_node::node_heartbeat))
        // Mapping & allocation routes
        .route("/allocations", get(handlers_mapping::list_allocations))
        .route("/mappings", get(handlers_mapping::list_mappings).post(handlers_mapping::create_mapping))
        .route("/mappings/:id", delete(handlers_mapping::delete_mapping))
        // Agent routes
        .route("/agent/:id/desired-state", get(handlers_agent::get_desired_state))
        .route("/agent/:id/report-state", post(handlers_agent::report_state))
        // Control & diagnostics
        .route("/reconcile", post(handlers_health::trigger_reconciliation))
        .route("/ports/status", get(handlers_health::get_ports_status))
        .layer(middleware::from_fn_with_state(auth_state, auth_middleware));

    Router::new()
        .route("/health", get(handlers_health::health_check))
        .nest("/api/v1", api_routes)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.trim());

    if let Some(token) = auth_header {
        if constant_time_eq(token, &state.api_key) {
            return Ok(next.run(request).await);
        }

        // Also allow valid agent tokens to query agent-specific routes
        if let Ok(nodes) = state.db.list_nodes() {
            for node in nodes {
                if constant_time_eq(token, &node.agent_token) {
                    return Ok(next.run(request).await);
                }
            }
        }
    }

    let err_resp = (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": "Unauthorized",
            "message": "Invalid or missing Bearer token"
        })),
    )
        .into_response();

    Err(err_resp)
}
